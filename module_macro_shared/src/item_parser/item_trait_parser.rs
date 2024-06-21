use std::path::PathBuf;
use syn::ItemTrait;

use crate::item_parser::ItemParser;
use crate::module_tree::Trait;
use crate::parse_container::ParseContainer;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use codegen_utils::syn_helper::SynHelper;
use crate::{BuildParseContainer, ItemModifier, logger_lazy, ModuleParser, ParseContainerItemUpdater, ParseContainerModifier, ProfileTreeFinalizer};
import_logger!("item_trait_parser.rs");

pub struct ItemTraitParser;

impl ItemParser<ItemTrait> for ItemTraitParser {
    fn parse_item<
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
    >(
        program_src: &PathBuf,
        parse_container: &mut ParseContainer,
        trait_found: &mut ItemTrait,
        mut path_depth: Vec<String>,
        module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT,
            ItemModifierT,
            ParseContainerModifierT,
            BuildParseContainerT,
            ParseContainerFinalizerT
        >,
    ) {
        info!("Adding trait: {:?}", SynHelper::get_str(&trait_found));
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
