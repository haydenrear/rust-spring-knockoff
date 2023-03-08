use std::any::Any;
use std::borrow::BorrowMut;
use std::cmp::Ordering;
use std::default::Default;
use std::env;
use std::fmt::{Debug, Formatter};
use std::io::Error;
use std::ops::{Deref, DerefMut};
use std::process::id;
use std::str::FromStr;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Block, FnArg, Item, ItemFn, parse, PatType, Stmt, Type};
use codegen_utils::syn_helper::{debug_struct_field_opt_tokens, SynHelper};
use knockoff_logging::use_logging;
use web_framework_shared::matcher::{AntStringRequestMatcher, Matcher};
use crate::parser::{CodegenItem, CodegenItemType, LibParser};

use_logging!();

use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

#[derive(Clone, Default)]
pub struct MethodAdviceAspectCodegen {
    pub default: Option<TokenStream>,
    pub item: Option<Item>,
    pub before_advice: Option<Block>,
    pub after_advice: Option<Block>,
    pub pointcut: PointCut,
    pub proceed_statement: Option<Stmt>,
    pub order: usize
}

impl Debug for MethodAdviceAspectCodegen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("MethodAdviceAspectCodegen");
        debug_struct_field_opt_tokens(&mut debug_struct, &self.after_advice, "after_advice");
        debug_struct_field_opt_tokens(&mut debug_struct, &self.before_advice, "before_advice");
        debug_struct_field_opt_tokens(&mut debug_struct, &self.default, "default");
        debug_struct_field_opt_tokens(&mut debug_struct, &self.proceed_statement, "proceed_statement");
        debug_struct_field_opt_tokens(&mut debug_struct, &self.item, "item");
        debug_struct.field("order", &self.order)
            .field("pointcut", &self.pointcut)
            .finish()
    }
}

impl Eq for MethodAdviceAspectCodegen {}

impl PartialEq<Self> for MethodAdviceAspectCodegen {
    fn eq(&self, other: &Self) -> bool {
        self.order.eq(&other.order)
    }
}

impl PartialOrd<Self> for MethodAdviceAspectCodegen {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl Ord for MethodAdviceAspectCodegen {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!()
    }
}

#[derive(Clone, Default)]
pub struct ParsedAspects {
    pub method_advice_aspects: Vec<MethodAdviceAspectCodegen>
}

#[derive(Clone, Default, Debug)]
pub struct PointCut {
    pub pointcut_expr: AntStringRequestMatcher,
}

impl PointCut {
    fn new(pointcut_expr_string: String) -> Self {
        let pointcut_expr = AntStringRequestMatcher::new(pointcut_expr_string, ".".to_string());
        Self {
            pointcut_expr
        }
    }
}

impl MethodAdviceAspectCodegen {

    pub(crate) fn create_aspect(path: &PointCut, ident: Ident) -> AspectMatcher {
        AspectMatcher::new(path, ident)
    }

    pub fn aspect_matches(bean_path: &Vec<String>, pointcut: &PointCut, bean_id: &String) -> bool {
        let pointcut_expr = pointcut.pointcut_expr.clone();
        let mut bean_path_with_id = bean_path.clone();
        bean_path_with_id.push(bean_id.clone());
        pointcut_expr.matches(bean_path_with_id.join(".").as_str())
    }

    pub(crate) fn new(item: &Item) -> Option<Self> {
        Some(
            match item {
                Item::Fn(item_fn) => {
                    let (proceed_statement, mut pointcut_expr, order) = Self::get_aspect_metadata(item_fn);

                    pointcut_expr = pointcut_expr.map(|mut p| {
                        p.replace(" ", "")
                    });

                    log_message!("{} is the pointcut expression of length {}.",
                        pointcut_expr.clone().unwrap(), pointcut_expr.clone().unwrap().len()
                    );

                    log_message!("here is the block for aspect: {}.", SynHelper::get_str(item_fn.block.clone()));

                    Self {
                        default: None,
                        item: Some(item.clone()),
                        before_advice: Some(Self::up_until_join_point(item_fn.block.deref())),
                        after_advice: Some(Self::after_join_point(item_fn.block.deref())),
                        pointcut: PointCut::new(pointcut_expr.unwrap()),
                        proceed_statement,
                        order,
                    }
                }
                _ => {
                    Self::default()
                }
            }
        )
    }

    fn get_aspect_metadata(item_fn: &ItemFn) -> (Option<Stmt>, Option<String>, usize) {
        let proceed_statement = item_fn.block.stmts.iter()
            .filter(|b| b.to_token_stream().to_string().as_str().contains("proceed"))
            .map(|b| b.clone())
            .next();

        let mut pointcut_expr = item_fn.attrs.iter()
            .filter(|a| a.to_token_stream().to_string().as_str().contains("aspect"))
            .map(|aspect_attr| SynHelper::parse_attr_path_single(aspect_attr))
            .next()
            .unwrap();

        let mut order = item_fn.attrs.iter()
            .filter(|a| a.to_token_stream().to_string().as_str().contains("ordered"))
            .map(|aspect_attr| SynHelper::parse_attr_path_single(aspect_attr)
                .map(|f| usize::from_str(f.as_str())
                    .or::<usize>(Ok(0)).unwrap())
            )
            .next()
            .unwrap()
            .or(Some(0))
            .unwrap();
        (proceed_statement, pointcut_expr, order)
    }

    fn is_aspect(vec: &Vec<Attribute>) -> bool {
        vec.iter()
            .any(|attr|
                attr.to_token_stream().to_string().as_str().contains("aspect")
            )
    }

    fn up_until_join_point(block: &Block) -> Block {
        let mut block_stmts = vec![];
        for stmt in &block.stmts {
            if stmt.to_token_stream().to_string().as_str().contains("proceed") {
                log_message!("Found proceed!");
                log_message!("Here are the before statements: ");
                Self::log_statements(&mut block_stmts);
                return Block {
                    brace_token: Default::default(),
                    stmts: block_stmts,
                };
            }
            log_message!("Adding {} statement", SynHelper::get_str(stmt));
            block_stmts.push(stmt.clone());
        }
        log_message!("Here are the before statements: ");
        Self::log_statements(&mut block_stmts);
        Block {
           brace_token: Default::default(),
           stmts: block_stmts
        }
    }

    fn after_join_point(block: &Block) -> Block {
        let mut block_stmts = vec![];
        let mut did_proceed = false;
        for stmt in &block.stmts {
            if stmt.to_token_stream().to_string().as_str().contains("proceed") {
                did_proceed = true;
            }
            if !did_proceed {
                continue;
            }
            if !stmt.to_token_stream().to_string().as_str().contains("proceed") {
                log_message!("Adding after statement: {}.", SynHelper::get_str(&stmt));
                block_stmts.push(stmt.clone());
            }
        }
        log_message!("Here are the after statements: ");
        Self::log_statements(&mut block_stmts);
        Block {
            brace_token: Default::default(),
            stmts: block_stmts
        }
    }

    fn log_statements(block_stmts: &mut Vec<Stmt>) {
        block_stmts.iter().for_each(|b| {
            log_message!("Next statement: {}.", SynHelper::get_str(b));
        });
    }
}

/// Aspect matcher matches all structs/impls in particular modules and packages, and allows for
/// matching based on the struct name.
pub(crate) struct AspectMatcher {
    module_path: AntStringRequestMatcher,
    struct_path: Ident,
}

impl AspectMatcher {
    fn new(module_path: &PointCut, struct_path: Ident) -> Self {
        Self {
            module_path: module_path.pointcut_expr.clone(),
            struct_path,
        }
    }
}

impl ParsedAspects {

    pub(crate) fn new(item: &Vec<Item>) -> Option<Self> {
        if item.len() > 0 {
            let vec = item.iter()
                .filter(|i| MethodAdviceAspectCodegen::supports_item(i))
                .map(|i| MethodAdviceAspectCodegen::new(i))
                .flatten()
                .collect::<Vec<MethodAdviceAspectCodegen>>();
            return Some(
                Self {
                    method_advice_aspects: vec
                }
            );
        }
        None
    }

    pub(crate) fn new_dyn_codegen(item: &Vec<Item>) -> Option<CodegenItemType> {
        Self::new(item)
            .map(|i| CodegenItemType::Aspect(i))
    }

}

impl CodegenItem for ParsedAspects {
    fn supports_item(item: &Vec<Item>) -> bool where Self: Sized {
        item.iter().any(|i| MethodAdviceAspectCodegen::supports_item(i))
    }

    fn supports(&self, item: &Vec<Item>) -> bool {
        item.iter().any(|i| MethodAdviceAspectCodegen::supports_item(i))
    }

    fn get_codegen(&self) -> Option<String> {
        if self.method_advice_aspects.len() == 0 {
            return None;
        }

        Some(
            self.method_advice_aspects.iter()
                .map(|m| m.get_codegen())
                .flatten()
                .collect::<Vec<String>>()
                .join("")
        )
    }

    fn default_codegen(&self) -> String {
        String::default()
    }

    fn get_unique_id(&self) -> String {
        "ParsedAspects".to_string()
    }
}

impl MethodAdviceAspectCodegen {

    fn supports_item(item: &Item) -> bool where Self: Sized {
        match item {
            Item::Fn(item_fn) => {
                Self::is_aspect(&item_fn.attrs)
            }
            Item::Mod(mod_aspect) => {
                Self::is_aspect(&mod_aspect.attrs)
            }
            _ => {
                false
            }
        }
    }

    fn supports(&self, item: &Item) -> bool {
        match item {
            Item::Fn(item_fn) => {
                MethodAdviceAspectCodegen::is_aspect(&item_fn.attrs)
            }
            _ => {
                false
            }
        }
    }

    fn get_codegen(&self) -> Option<String> {
        None
    }

    fn default_codegen(&self) -> String {
        String::default()
    }

    fn get_unique_id(&self) -> String {
        "MethodAdviceAspect".to_string()
    }

}


#[derive(Default, Clone)]
pub struct AspectParser {
    pub aspects: Vec<ParsedAspects>,
}

impl AspectParser {

    pub fn parse_method_advice_aspects() -> Vec<ParsedAspects> {
        log_message!("Parsing aspects.");
        env::var("KNOCKOFF_FACTORIES").map(|aug_file| {
            log_message!("Found knockoff factories file {}. Parsing aspects.", aug_file.as_str());
            LibParser::parse_codegen_items(&aug_file)
                .iter()
                .flat_map(|c|{
                    match c {
                        CodegenItemType::Aspect(aspect) => {
                            vec![aspect.clone()]
                        }
                        _ => {
                           vec![]
                        }
                    }
                })
                .collect::<Vec<ParsedAspects>>()
        }).ok().or(Some(vec![])).unwrap()
    }

    pub fn new_aspects() -> Self {
        Self::parse_aspects()
    }

    pub fn parse_aspects() -> Self {
        Self {
            aspects: Self::parse_method_advice_aspects()
        }
    }

}

