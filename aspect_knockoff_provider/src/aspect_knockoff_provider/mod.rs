use std::any::Any;
use std::borrow::BorrowMut;
use std::default::Default;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use proc_macro2::Ident;
use quote::{TokenStreamExt, ToTokens};
use rand::Rng;
use syn::{Block, ImplItemMethod, ItemImpl, Stmt, Type};
use aspect_parse_provider::MethodAdviceAspectCodegen;
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::impl_parse_values;
use module_macro_shared::parse_container::{MetadataItem, ParseContainer};
use web_framework_shared::matcher::{AntStringRequestMatcher, Matcher};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("provider.rs");

pub mod aspect_parse_provider;
pub mod aspect_ts_generator;
pub mod aspect_item_modifier;
pub mod debug;

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

#[derive(Default, Clone)]
pub struct MethodAdviceChain {
    pub before_advice: Option<Block>,
    pub after_advice: Option<Block>,
    pub proceed_statement: Option<Stmt>
}

impl MethodAdviceChain {
    pub fn new(method_advice: &MethodAdviceAspectCodegen) -> Self {
        Self {
            before_advice: method_advice.before_advice.clone(),
            after_advice: method_advice.after_advice.clone(),
            proceed_statement: method_advice.proceed_statement.clone(),
        }
    }
}

#[derive(Default, Clone)]
pub struct AspectInfo {
    pub method_advice_aspect: MethodAdviceAspectCodegen,
    pub method: Option<ImplItemMethod>,
    pub args: Vec<(Ident, Type)>,
    /// This is the block before any aspects are added.
    pub original_fn_logic: Option<Block>,
    pub return_type: Option<Type>,
    pub method_after: Option<ImplItemMethod>,
    pub mutable: bool,
    pub advice_chain: Vec<MethodAdviceChain>,
}

impl MetadataItem for AspectInfo {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl_parse_values!(AspectInfo);

pub fn matches_ignore_traits(matches_ignore_traits: &str) -> bool {
    vec!["Default", "Debug"].iter().any(|i| matches_ignore_traits.contains(i))
}

pub fn is_ignore_trait(item_impl: &ItemImpl) -> bool {
    if item_impl.trait_.as_ref().filter(|t| matches_ignore_traits(&SynHelper::get_str(t.1.to_token_stream().to_string().as_str())))
        .is_some() {
        log_message!("Ignoring {}.", SynHelper::get_str(&item_impl));
        return true;
    }
    false
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


