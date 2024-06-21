use std::path::PathBuf;
use std::sync::Mutex;

use syn::__private::str;
use syn::Item;
use syn::parse::Parser;

use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::*;

use crate::{BuildParseContainer, ItemModifier, ItemParser, ModuleParser, ParseContainer, ParseContainerItemUpdater, ParseContainerModifier, ProfileTreeFinalizer};
use crate::item_mod_parser::ItemModParser;
use crate::logger_lazy;

import_logger!("module_parser.rs");


pub fn parse_module_into_container<
    ParseContainerItemUpdaterT: ParseContainerItemUpdater,
    ItemModifierT: ItemModifier,
    ParseContainerModifierT: ParseContainerModifier,
    BuildParseContainerT: BuildParseContainer,
    ParseContainerFinalizerT: ProfileTreeFinalizer,
>(
    program_src: &PathBuf,
    mut found: &mut Item,
    module_parser:
    &mut ModuleParser<
        ParseContainerItemUpdaterT,
        ItemModifierT,
        ParseContainerModifierT,
        BuildParseContainerT,
        ParseContainerFinalizerT
    >,
) -> Option<ParseContainer>
    where
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
{
    create_initial_parse_container(program_src, &mut found, module_parser)
        .map(|mut created| {
             do_container_modifications(&mut found, created, module_parser)
        })
}

pub fn do_modify<
    ParseContainerItemUpdaterT: ParseContainerItemUpdater,
    ItemModifierT: ItemModifier,
    ParseContainerModifierT: ParseContainerModifier,
    BuildParseContainerT: BuildParseContainer,
    ParseContainerFinalizerT: ProfileTreeFinalizer,
>(mut found: &mut Item, mut created: &mut ParseContainer, path: Vec<String>, module_parser: &mut ModuleParser<
    ParseContainerItemUpdaterT,
    ItemModifierT,
    ParseContainerModifierT,
    BuildParseContainerT,
    ParseContainerFinalizerT
>){
    ItemModifierT::modify_item(&mut created, &mut found, path);
}

pub fn do_container_modifications<
    ParseContainerItemUpdaterT: ParseContainerItemUpdater,
    ItemModifierT: ItemModifier,
    ParseContainerModifierT: ParseContainerModifier,
    BuildParseContainerT: BuildParseContainer,
    ParseContainerFinalizerT: ProfileTreeFinalizer,
>(mut found: &mut Item, mut created: (ParseContainer, String), module_parser: &mut ModuleParser<
    ParseContainerItemUpdaterT,
    ItemModifierT,
    ParseContainerModifierT,
    BuildParseContainerT,
    ParseContainerFinalizerT
>) -> ParseContainer {
    let mut container = created.0;
    ItemModifierT::modify_item(&mut container, &mut found, vec![created.1.clone()]);
    container
}

pub(crate) fn create_initial_parse_container<
    'a,
    ParseContainerItemUpdaterT: ParseContainerItemUpdater,
    ItemModifierT: ItemModifier,
    ParseContainerModifierT: ParseContainerModifier,
    BuildParseContainerT: BuildParseContainer,
    ParseContainerFinalizerT: ProfileTreeFinalizer,
>(
    program_src: &PathBuf,
    mut found: &mut Item,
    module_parser: &mut ModuleParser<
        ParseContainerItemUpdaterT,
        ItemModifierT,
        ParseContainerModifierT,
        BuildParseContainerT,
        ParseContainerFinalizerT
    >,
) -> Option<(ParseContainer, String)> {
    let mut created = match &mut found {
        Item::Mod(ref mut module_found) => {
            let mut container = ParseContainer::default();
            info!("Doing parse, of {:?}", SynHelper::get_str(&module_found));

            ItemModParser::parse_item(
                program_src,
                &mut container,
                module_found,
                vec![module_found.ident.to_string().clone()],
                module_parser
            );

            info!("{} is module and {:?} is container after item mod parsing.",
                module_found.ident.to_string().as_str(), &container);

            Some((container, module_found.ident.to_string().clone()))
        }
        _ => {
            None
        }
    };
    created
}

pub fn do_parse_container<
    'a,
    ParseContainerItemUpdaterT: ParseContainerItemUpdater,
    ItemModifierT: ItemModifier,
    ParseContainerModifierT: ParseContainerModifier,
    BuildParseContainerT: BuildParseContainer,
    ParseContainerFinalizerT: ProfileTreeFinalizer,
>(
    program_src: &PathBuf,
    mut found: &mut Item,
    module_parser: &mut ModuleParser<
        ParseContainerItemUpdaterT,
        ItemModifierT,
        ParseContainerModifierT,
        BuildParseContainerT,
        ParseContainerFinalizerT
    >,
    mut container: &mut ParseContainer,
) {
    match &mut found {
        Item::Mod(ref mut module_found) => {
            info!("Doing parse, of {:?}", SynHelper::get_str(&module_found));

            ItemModParser::parse_item(
                program_src,
                &mut container,
                module_found,
                vec![module_found.ident.to_string().clone()],
                module_parser
            );

            info!("{} is module and {:?} is container after item mod parsing.",
                module_found.ident.to_string().as_str(), &container);

            Some((container, module_found.ident.to_string().clone()))
        }
        _ => {
            None
        }
    };
}
