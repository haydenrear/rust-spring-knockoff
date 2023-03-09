use std::any::Any;
use std::ops::Deref;
use std::path::Path;
use std::thread::available_parallelism;
use paste::item;
use quote::ToTokens;
use syn::{Fields, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait};
use codegen_utils::syn_helper::SynHelper;
use item_impl_parser::ItemImplParser;
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;

use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::module_tree::{AutowireType, Bean, ModulesFunctions, Trait};
use crate::module_macro_lib::parse_container::ParseContainer;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();

pub mod item_impl_parser;
pub mod item_enum_parser;
pub mod item_struct_parser;
pub mod item_mod_parser;
pub mod item_trait_parser;
pub mod item_fn_parser;
#[cfg(test)]
pub mod test;

pub trait ItemParser<T: ToTokens> {
    fn parse_item(parse_container: &mut ParseContainer, item: &mut T, path_depth: Vec<String>);
}
