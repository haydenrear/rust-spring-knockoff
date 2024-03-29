use syn::{FnArg, Generics, ItemFn, PatType, Type};
use std::fmt::{Debug, Formatter};
use proc_macro2::Ident;
use codegen_utils::syn_helper;
use quote::ToTokens;
use crate::bean::{BeanPath, BeanType};
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("functions.rs");
use crate::profile_tree::ProfileBuilder;

/**
Will be annotated with #[bean] and #[singleton], #[prototype] as provided factory functions.
 **/
#[derive(Clone)]
pub struct ModulesFunctions {
    pub fn_found: FunctionType,
    pub path: Vec<String>,
    pub id: String
}

#[derive(Clone)]
pub struct FunctionType {
    pub item_fn: ItemFn,
    pub qualifiers: Vec<String>,
    pub profiles: Vec<ProfileBuilder>,
    pub fn_type: Option<BeanPath>,
    pub bean_type: BeanType,
    pub args: Vec<(Ident, BeanPath, Option<String>, PatType)>,
    pub output_generics: Generics,
    pub fn_arg_generics: Generics
}

impl Debug for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let debug_struct = &mut f.debug_struct("FunctionType");
        debug_struct.field("qualifier", &self.qualifiers);
        self.fn_type.as_ref().map(|fn_type| {
            debug_struct.field("fn_type", fn_type);
        });
        debug_struct.field("item_fn", &self.item_fn.to_token_stream().to_string().as_str());
        debug_struct.field("bean_type", &self.bean_type);
        Ok(())
    }
}
