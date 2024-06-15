use syn::{Item, ItemMod, Path};
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
use codegen_utils::syn_helper::SynHelper;
use program_parser::module_iterator::ModuleIterator;
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
    >(
        parse_container: &mut ParseContainer,
        mut item_found: &mut ItemMod,
        mut path_depth: Vec<String>,
        module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT,
            ItemModifierT,
            ParseContainerModifierT,
            BuildParseContainerT,
            ParseContainerFinalizerT
        >) {

        let mut item_found = ModuleIterator::retrieve_next_mod(item_found.clone(), &std::path::Path::new("/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test/src").to_path_buf())
            .or(Some(item_found.clone())).unwrap();
        path_depth.push(item_found.ident.to_string().clone());

        info!("Parsing {:?}", SynHelper::get_str(item_found.ident.clone()));

        item_found.content.iter_mut()
            .flat_map(|mut c| c.1.iter_mut())
            .for_each(|i: &mut Item| {
                info!("Parsing {:?}", SynHelper::get_str(i.clone()));
                if let Item::Mod(item_mod) = i {
                    let mut item_mod = ModuleIterator::retrieve_next_mod(item_mod.clone(), &std::path::Path::new("/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test/src").to_path_buf())
                        .or(Some(item_mod.clone()))
                        .unwrap();
                    Self::parse_item(parse_container, &mut item_mod, path_depth.clone(), module_parser);
                } else {
                    ParseContainerItemUpdaterT::parse_update(i, parse_container);
                    Self::parse_item_inner(i, parse_container, &mut path_depth.clone(), module_parser);
                }
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
                info!("Found const val {}.", const_val.to_token_stream().clone());
            }
            Item::Enum(enum_type) => {
                info!("Found enum val {}.", enum_type.to_token_stream().clone());
                ItemEnumParser::parse_item(app_container, enum_type, path_depth.clone(), module_parser);
            }
            Item::Fn(fn_type) => {
                info!("Found fn type {}.", fn_type.to_token_stream().clone());

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
                info!("Found module with name {} !!!", module.ident.to_string().clone());
                ItemModParser::parse_item(app_container, module, path_depth.clone(), module_parser);
            }
            Item::Static(static_val) => {
                info!("Found static val {}.", static_val.to_token_stream().clone());
            }
            Item::Struct(ref mut item_struct) => {
                ItemStructParser::parse_item(app_container, item_struct, path_depth.clone(), module_parser);
            }
            Item::Trait(trait_created) => {
                info!("Trait created: {}", trait_created.ident.clone().to_string());
                ItemTraitParser::parse_item(app_container, trait_created, path_depth.clone(), module_parser);
            }
            Item::TraitAlias(_) => {}
            Item::Type(type_found) => {
                info!("Item type found {}!", type_found.ident.to_token_stream().to_string().clone());
            }
            Item::Union(_) => {}
            Item::Verbatim(_) => {}
            _ => {}
        }
    }
}
