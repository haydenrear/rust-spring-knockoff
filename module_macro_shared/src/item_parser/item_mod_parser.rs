use std::path::PathBuf;
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
use codegen_utils::{FlatMapOptional, project_directory};
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
        program_src: &PathBuf,
        parse_container: &mut ParseContainer,
        mut item_found: &mut ItemMod,
        mut path_depth: Vec<String>,
        module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT,
            ItemModifierT,
            ParseContainerModifierT,
            BuildParseContainerT,
            ParseContainerFinalizerT
        >
    ) {


        let mut item_found = ModuleIterator::retrieve_next_mod(item_found.clone(), program_src)
            .or(Some(item_found.clone())).unwrap();

        info!("Parsing {:?}", SynHelper::get_str(item_found.ident.clone()));

        item_found.content.iter_mut()
            .flat_map(|mut c| c.1.iter_mut())
            .for_each(|i: &mut Item| {
                info!("Parsing in item_mod parser {:?}", SynHelper::get_str(i.clone()));
                // probably best to retrieve a clone of the item, pass that in, and then replace it.
                ItemModifierT::modify_item(parse_container, i, path_depth.clone());
                ParseContainerItemUpdaterT::parse_update(i, parse_container);
                if let Item::Mod(item_mod) = i {
                    let mut item_mod = ModuleIterator::retrieve_next_mod(item_mod.clone(), program_src)
                        .or(Some(item_mod.clone()))
                        .unwrap();
                    let mut next_path_deps = path_depth.clone();
                    next_path_deps.push(item_mod.ident.to_string().clone());
                    Self::parse_item(program_src, parse_container, &mut item_mod, next_path_deps, module_parser);
                } else {
                    Self::parse_item_inner(program_src,i, parse_container, &mut path_depth.clone(), module_parser);
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
    >(
        program_src: &PathBuf,
        i: &mut Item,
        mut app_container: &mut ParseContainer,
        path_depth: &mut Vec<String>,
        module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT,
            ItemModifierT,
            ParseContainerModifierT,
            BuildParseContainerT,
            ParseContainerFinalizerT
        >,
    ) {
        let container_key = ParseContainer::get_bean_definition_key(&i);
        match i {
            Item::Const(const_val) => {
                info!("Found const val {}.", const_val.to_token_stream().clone());
            }
            Item::Enum(enum_type) => {
                info!("Found enum val {}.", enum_type.to_token_stream().clone());
                ItemEnumParser::parse_item(program_src, app_container, enum_type, path_depth.clone(), module_parser);
            }
            Item::Fn(fn_type) => {
                info!("Found fn type {}.", fn_type.to_token_stream().clone());
                let did_exist_bd = container_key.clone().flat_map_opt(|c| {
                    let mut bean_def = app_container.injectable_types_builder.remove(&c);

                    bean_def.as_mut()
                        .flat_map_opt(|bd| bd.factory_fn.as_mut())
                        .map(|m| ItemFnParser::parse_item(program_src, &mut app_container, &mut m.fn_found.item_fn, path_depth.clone(), module_parser));

                    bean_def.flat_map_opt(|bd| {
                            app_container.injectable_types_builder.insert(c.clone(), bd);
                            Some(true)
                        })
                        .or(Some(false))
                });

                if let None | Some(false) = did_exist_bd {
                    ItemFnParser::parse_item(program_src, &mut app_container, fn_type, path_depth.clone(), module_parser);
                }
            }
            Item::ForeignMod(_) => {}
            Item::Impl(ref mut impl_found) => {
                ItemImplParser::parse_item(program_src, app_container, impl_found, path_depth.clone(), module_parser);
            }
            Item::Macro(macro_created) => {
            }
            Item::Macro2(_) => {}
            Item::Mod(ref mut module) => {
                info!("Found module with name {} !!!", module.ident.to_string().clone());
                ItemModParser::parse_item(program_src, app_container, module, path_depth.clone(), module_parser);
            }
            Item::Static(static_val) => {
                info!("Found static val {}.", static_val.to_token_stream().clone());
            }
            Item::Struct(ref mut item_struct) => {
                ItemStructParser::parse_item(program_src, app_container, item_struct, path_depth.clone(), module_parser);
            }
            Item::Trait(trait_created) => {
                info!("Trait created: {}", trait_created.ident.clone().to_string());
                ItemTraitParser::parse_item(program_src, app_container, trait_created, path_depth.clone(), module_parser);
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
