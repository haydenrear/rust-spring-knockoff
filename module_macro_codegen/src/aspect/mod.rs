use std::any::Any;
use std::borrow::BorrowMut;
use std::default::Default;
use std::env;
use std::io::Error;
use std::ops::{Deref, DerefMut};
use std::process::id;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, Block, FnArg, Item, ItemFn, PatType, Stmt, Type};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::use_logging;
use web_framework_shared::matcher::{AntStringRequestMatcher, Matcher};
use crate::parser::{CodegenItem, LibParser};

use_logging!();

use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

#[derive(Clone, Default)]
pub struct MethodAdviceAspectCodegen {
    pub default: Option<TokenStream>,
    pub item: Option<Item>,
    pub type_args: Vec<Type>,
    pub before_advice: Option<Block>,
    pub after_advice: Option<Block>,
    pub pointcut: PointCut,
}

#[derive(Clone, Default)]
pub struct ParsedAspects {
    pub method_advice_aspects: Vec<MethodAdviceAspectCodegen>
}

#[derive(Clone, Default)]
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
    pub(crate) fn create_aspect_matcher(&self) -> AspectMatcher {
        Self::create_aspect(&self.pointcut, Ident::new("TestAspect", Span::call_site()))
    }

    pub(crate) fn create_aspect(path: &PointCut, ident: Ident) -> AspectMatcher {
        AspectMatcher::new(path, ident)
    }

    fn is_pointcut_arg(pat_type: &PatType) -> bool {
        let string = pat_type.ty.to_token_stream().to_string();
        let type_string = string.as_str();
        if !type_string.contains("JoinPoint") {
            return true;
        }
        false
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
                    let type_args = item_fn.sig.inputs.iter()
                        .flat_map(|i| {
                            match i {
                                FnArg::Receiver(_) => {
                                    vec![]
                                }
                                FnArg::Typed(typed) => {
                                    if Self::is_pointcut_arg(typed) {
                                        return vec![typed.ty.deref().clone()];
                                    }
                                    vec![]
                                }
                            }
                        }).collect();

                    let mut pointcut_expr = item_fn.attrs.iter()
                        .filter(|a| a.to_token_stream().to_string().as_str().contains("aspect"))
                        .map(|aspect_attr| SynHelper::parse_attr_path_single(aspect_attr))
                        .next()
                        .unwrap();

                    pointcut_expr = pointcut_expr.map(|mut p| {
                        p.replace(" ", "")
                    });

                    log_message!("{} is the pointcut expression of length {}.",
                        pointcut_expr.clone().unwrap(), pointcut_expr.clone().unwrap().len()
                    );

                    Self {
                        default: None,
                        item: Some(item.clone()),
                        type_args,
                        before_advice: Some(Self::up_until_join_point(item_fn.block.deref())),
                        after_advice: Some(Self::after_join_point(item_fn.block.deref())),
                        pointcut: PointCut::new(pointcut_expr.unwrap()),
                    }
                }
                _ => {
                    Self::default()
                }
            }
        )
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
            if stmt.to_token_stream().to_string().as_str().contains("proceed()") {
                return Block {
                    brace_token: Default::default(),
                    stmts: block_stmts,
                };
            }
            block_stmts.push(stmt.clone());
        }
        Block {
           brace_token: Default::default(),
           stmts: block_stmts
        }
    }

    fn after_join_point(block: &Block) -> Block {
        let mut block_stmts = vec![];
        let mut did_proceed = false;
        for stmt in &block.stmts {
            if stmt.to_token_stream().to_string().as_str().contains("proceed()") {
                did_proceed = true;
            }
            if !did_proceed {
                continue;
            }
            if !stmt.to_token_stream().to_string().as_str().contains("proceed()") {
                block_stmts.push(stmt.clone());
            }
        }
        Block {
            brace_token: Default::default(),
            stmts: block_stmts
        }
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
            return Some(
                Self {
                    method_advice_aspects: item.iter()
                        .filter(|i| MethodAdviceAspectCodegen::supports_item(i))
                        .map(|i| MethodAdviceAspectCodegen::new(i))
                        .flatten()
                        .collect::<Vec<MethodAdviceAspectCodegen>>()
                }
            );
        }
        None
    }

    pub(crate) fn new_dyn_codegen(item: &Vec<Item>) -> Option<Box<dyn CodegenItem>> {
        Self::new(item)
            .map(|i| Box::new(i) as Box<dyn CodegenItem>)
    }

    pub(crate) fn new_any(item: &Vec<Item>) -> Option<Box<dyn Any>> {
        Self::new(item)
            .map(|i| Box::new(i) as Box<dyn Any>)
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

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        Box::new(ParsedAspects::default())
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
            LibParser::parse_codegen_items_any(&aug_file)
                .iter()
                .flat_map(|c| c.downcast_ref::<ParsedAspects>()
                    .map(|c| {
                        log_message!("Downcasted method advice aspect.");
                        c
                    })
                    .map(|m| vec![m]).or(Some(vec![]))
                    .unwrap()
                )
                .flat_map(|b| {
                    log_message!("Found method advice aspect.");
                    vec![b.clone()]
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

    pub fn write_aspect(&self, type_for_aspect: Type, args_for_aspect: Option<Type>, aspect_fn: TokenStream) -> TokenStream {
        if args_for_aspect.is_some() {
            let args = args_for_aspect.unwrap();
            quote! {
                impl AspectWithArgs<#args> for #type_for_aspect {
                    #aspect_fn
                }
            }
        } else {
            quote! {
                impl Aspect for #type_for_aspect {
                    #aspect_fn
                }
            }
        }
    }
}

