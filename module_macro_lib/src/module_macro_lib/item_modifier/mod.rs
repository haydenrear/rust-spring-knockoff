use std::ops::Deref;
use proc_macro2::{Ident, Span};
use quote::{quote_spanned, ToTokens};
use syn::{Block, FnArg, ImplItem, ImplItemMethod, Item, ItemImpl, parse, Pat, ReturnType, Type};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use crate::module_macro_lib::parse_container::ParseContainer;

use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;
use module_macro_shared::aspect::AspectInfo;
use crate::module_macro_lib::item_modifier::aspect_modifier::AspectModifier;

pub mod aspect_modifier;
pub mod delegating_modifier;

pub trait ItemModifier {
    fn modify_item(&self, parse_container: &mut ParseContainer, item: &mut Item, path_depth: Vec<String>);
    fn supports_item(&self, item: &Item) -> bool;
}
