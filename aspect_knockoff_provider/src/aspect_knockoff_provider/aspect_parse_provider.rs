use proc_macro2::{Ident, TokenStream};
use syn::{Attribute, Block, Item, ItemFn, Stmt};
use codegen_utils::syn_helper::{debug_struct_field_opt_tokens, SynHelper};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use quote::ToTokens;
use std::ops::Deref;
use std::str::FromStr;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use module_macro_shared::parse_container::{MetadataItemId, ParseContainer};
use web_framework_shared::matcher::Matcher;
use crate::aspect_knockoff_provider::{AspectMatcher, PointCut};

use module_macro_shared::impl_parse_values;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("aspect_parse_provider.rs");


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

impl MetadataItem for MethodAdviceAspectCodegen {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl Debug for MethodAdviceAspectCodegen {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // let mut debug_struct = f.debug_struct("MethodAdviceAspectCodegen");
        // debug_struct_field_opt_tokens(&mut debug_struct, &self.after_advice, "after_advice");
        // debug_struct_field_opt_tokens(&mut debug_struct, &self.before_advice, "before_advice");
        // debug_struct_field_opt_tokens(&mut debug_struct, &self.default, "default");
        // debug_struct_field_opt_tokens(&mut debug_struct, &self.proceed_statement, "proceed_statement");
        // debug_struct_field_opt_tokens(&mut debug_struct, &self.item, "item");
        // debug_struct.field("order", &self.order)
        //     .field("pointcut", &self.pointcut)
        //     .finish()
        Ok(())
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
pub struct AspectGeneratorMutableModifier ;

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

    pub(crate) fn new(item: &Item) -> Self {
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
    }

    fn get_aspect_metadata(item_fn: &ItemFn) -> (Option<Stmt>, Option<String>, usize) {
        let proceed_statement = item_fn.block.stmts.iter()
            .filter(|b| b.to_token_stream().to_string().as_str().contains("proceed"))
            .map(|b| {
                log_message!("{} is the proceed statement.", SynHelper::get_str(b));
                b.clone()
            })
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

    pub(crate) fn is_aspect(vec: &Vec<Attribute>) -> bool {
        vec.iter()
            .any(|attr| attr.to_token_stream().to_string().as_str().contains("aspect"))
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

    pub(crate) fn supports_item(item: &Item) -> bool where Self: Sized {
        let supports = match item {
            Item::Fn(item_fn) => {
                Self::is_aspect(&item_fn.attrs)
            }
            Item::Mod(mod_aspect) => {
                Self::is_aspect(&mod_aspect.attrs)
            }
            _ => {
                false
            }
        };
        if supports {
            info!("Found supporting: {:?}", SynHelper::get_str(item));
        }
        supports
    }
}

#[derive(Clone, Default)]
pub struct ParsedAspects {
    pub method_advice_aspects: Vec<MethodAdviceAspectCodegen>
}

use module_macro_shared::parse_container::MetadataItem;
use std::rc::Rc;
use std::any::Any;
use collection_util::add_to_multi_value;

use std::sync::Arc;

impl_parse_values!(MethodAdviceAspectCodegen);

/// Generates the token stream from the aspects created, if any.
impl ParsedAspects {

    /// Add the method advice aspects initially to the container. This is required to happen in a first
    /// pass of the complete program because of what aspects **do**.
    pub fn parse_update(items: &mut Item, parse_container: &mut ParseContainer) {
        info!("In parse update.");
        if MethodAdviceAspectCodegen::supports_item(items) {
            info!("Found aspect {:?} while parsing container.",
                items.to_token_stream().to_string().as_str());
            let new_method = MethodAdviceAspectCodegen::new(items);
            let metadata = MetadataItemId::new(
                "".to_string(), "MethodAdviceAspectCodegen".to_string());
            add_to_multi_value(&mut parse_container.provided_items,
                               Box::new(new_method), metadata);
            info!("Added to provided: {}.", parse_container.provided_items.len());
        }
    }

    fn supports_item(item: &Vec<Item>) -> bool where Self: Sized {
        item.iter().any(|i| MethodAdviceAspectCodegen::supports_item(i))
    }

    fn supports(&self, item: &Vec<Item>) -> bool {
        item.iter().any(|i| MethodAdviceAspectCodegen::supports_item(i))
    }

}
