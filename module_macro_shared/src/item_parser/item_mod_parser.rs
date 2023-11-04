use syn::{Item, ItemMod};
use crate::item_parser::item_enum_parser::ItemEnumParser;
use crate::item_parser::item_fn_parser::ItemFnParser;
use crate::item_parser::item_impl_parser::ItemImplParser;
use crate::item_parser::item_struct_parser::ItemStructParser;
use crate::item_parser::item_trait_parser::ItemTraitParser;
use crate::item_parser::ItemParser;
use crate::parse_container::ParseContainer;

use quote::ToTokens;
use crate::parse_container::ParseContainerItemUpdater;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::{BuildParseContainer, ItemModifier, logger_lazy, ModuleParser, ParseContainerModifier, ProfileTreeFinalizer};
import_logger!("item_mod_parser.rs");


pub struct ItemModParser;

impl ItemParser<ItemMod> for ItemModParser {
    fn parse_item<
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
    >(parse_container: &mut ParseContainer,
                  item_found: &mut ItemMod, mut path_depth: Vec<String>, module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT,
            ItemModifierT,
            ParseContainerModifierT,
            BuildParseContainerT,
            ParseContainerFinalizerT
        >) {
        path_depth.push(item_found.ident.to_string().clone());
        item_found.content.iter_mut()
            .flat_map(|mut c| c.1.iter_mut())
            .for_each(|i: &mut Item| {
                ParseContainerItemUpdaterT::parse_update(i, parse_container);
                Self::parse_item_inner(i, parse_container, &mut path_depth.clone(), module_parser);
            });
    }
}

impl ItemModParser {
    pub fn parse_item_inner<
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
    >(i: &mut Item, mut app_container: &mut ParseContainer, path_depth: &mut Vec<String>, module_parser: &mut ModuleParser<
        ParseContainerItemUpdaterT,
        ItemModifierT,
        ParseContainerModifierT,
        BuildParseContainerT,
        ParseContainerFinalizerT
    >) {
        match i {
            Item::Const(const_val) => {
                log_message!("Found const val {}.", const_val.to_token_stream().clone());
            }
            Item::Enum(enum_type) => {
                log_message!("Found enum val {}.", enum_type.to_token_stream().clone());
                ItemEnumParser::parse_item(app_container, enum_type, path_depth.clone(), module_parser);
            }
            Item::Fn(fn_type) => {
                log_message!("Found fn type {}.", fn_type.to_token_stream().clone());

                ItemFnParser::parse_item(app_container, fn_type, path_depth.clone(), module_parser);
            }
            Item::ForeignMod(_) => {}
            Item::Impl(ref mut impl_found) => {
                ItemImplParser::parse_item(app_container, impl_found, path_depth.clone(), module_parser);
            }
            Item::Macro(macro_created) => {
            }
            Item::Macro2(_) => {}
            Item::Mod(ref mut module) => {
                log_message!("Found module with name {} !!!", module.ident.to_string().clone());
                ItemModParser::parse_item(app_container, module, path_depth.clone(), module_parser);
            }
            Item::Static(static_val) => {
                log_message!("Found static val {}.", static_val.to_token_stream().clone());
            }
            Item::Struct(ref mut item_struct) => {
                ItemStructParser::parse_item(app_container, item_struct, path_depth.clone(), module_parser);
            }
            Item::Trait(trait_created) => {
                log_message!("Trait created: {}", trait_created.ident.clone().to_string());
                ItemTraitParser::parse_item(app_container, trait_created, path_depth.clone(), module_parser);
            }
            Item::TraitAlias(_) => {}
            Item::Type(type_found) => {
                log_message!("Item type found {}!", type_found.ident.to_token_stream().to_string().clone());
            }
            Item::Union(_) => {}
            Item::Verbatim(_) => {}
            _ => {}
        }
    }
}
