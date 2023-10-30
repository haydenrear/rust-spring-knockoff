use syn::ItemTrait;

use crate::module_macro_lib::item_parser::ItemParser;
use module_macro_shared::module_tree::Trait;
use module_macro_shared::parse_container::ParseContainer;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("item_trait_parser.rs");

pub struct ItemTraitParser;

impl ItemParser<ItemTrait> for ItemTraitParser {
    fn parse_item(parse_container: &mut ParseContainer, trait_found: &mut ItemTrait, mut path_depth: Vec<String>) {
        path_depth.push(trait_found.ident.to_string().clone());
        if !parse_container.traits.contains_key(&trait_found.ident.to_string().clone()) {
            parse_container.traits.insert(
                trait_found.ident.to_string().clone(),
                Trait::new(trait_found.clone(), path_depth)
            );
        } else {
            log_message!("Contained trait already!");
        }
    }
}
