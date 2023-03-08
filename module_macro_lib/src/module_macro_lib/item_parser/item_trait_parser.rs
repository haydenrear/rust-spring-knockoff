use syn::ItemTrait;

use crate::module_macro_lib::item_parser::ItemParser;
use crate::module_macro_lib::module_tree::Trait;
use crate::module_macro_lib::parse_container::ParseContainer;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

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
