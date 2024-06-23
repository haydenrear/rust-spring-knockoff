use std::path::PathBuf;
use std::sync::Mutex;

use quote::ToTokens;
use syn::{Item, ItemMod};
use codegen_utils::FlatMapOptional;

use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::*;
use program_parser::module_iterator::ModuleIterator;
use program_parser::module_locator::is_in_line_module;

use crate::{BuildParseContainer, do_parse_container, do_parse_container_item_mod, ItemModifier, logger_lazy, ModuleParser, ParseContainerModifier, ProfileTreeFinalizer};
use crate::item_parser::item_enum_parser::ItemEnumParser;
use crate::item_parser::item_fn_parser::ItemFnParser;
use crate::item_parser::item_impl_parser::ItemImplParser;
use crate::item_parser::item_struct_parser::ItemStructParser;
use crate::item_parser::item_trait_parser::ItemTraitParser;
use crate::item_parser::ItemParser;
use crate::parse_container::ParseContainer;
use crate::parse_container::ParseContainerItemUpdater;

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
        let module_key = ParseContainer::get_bean_definition_key_item_mod(&item_found).unwrap();
        parse_container.modules.remove(&module_key).as_mut()
            .map(|item_found| { Self::do_parse_module(program_src, parse_container, &mut path_depth, module_parser, item_found); })
            .or_else(|| {
                if is_in_line_module(item_found) {
                    info!("Parsing in-line module: {:?}.", SynHelper::get_str(&item_found.ident));
                    Self::do_parse_module(program_src, parse_container, &mut path_depth, module_parser, item_found);
                } else {
                    info!("Retrieving in-line module: {:?}.", SynHelper::get_str(&item_found.ident));
                    ModuleIterator::retrieve_next_mod(item_found.clone(), &program_src).as_mut()
                        .map(|item_found| {
                            info!("Found in-line module from module: {:?}.", SynHelper::get_str(&item_found.ident));
                            Self::do_parse_module(program_src, parse_container, &mut path_depth, module_parser, item_found);
                        });
                }
                None
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
                ItemFnParser::parse_item(program_src, &mut app_container, fn_type, path_depth.clone(), module_parser);
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

    fn do_parse_module<
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
    >(
        program_src: &PathBuf,
        parse_container: &mut ParseContainer,
        mut path_depth: &mut Vec<String>,
        module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT, ItemModifierT, ParseContainerModifierT,
            BuildParseContainerT, ParseContainerFinalizerT
        >,
        item_found: &mut ItemMod
    ) {
        info!("Parsing {:?}", SynHelper::get_str(item_found.ident.clone()));
        item_found.content.iter_mut()
            .flat_map(|mut c| c.1.iter_mut())
            .for_each(|next_item: &mut Item| {
                info!("Parsing in item_mod parser {:?}", SynHelper::get_str(next_item.clone()));
                // probably best to retrieve a clone of the item, pass that in, and then replace it.
                if let Item::Mod(item_mod) = next_item {
                    Self::parse_sub_module(program_src, parse_container, path_depth, module_parser, item_mod);
                } else {
                    ItemModifierT::modify_item(parse_container, next_item, path_depth.clone());
                    ParseContainerItemUpdaterT::parse_update(next_item, parse_container);
                    Self::parse_item_inner(program_src, next_item, parse_container, &mut path_depth.clone(), module_parser);
                }
            });

        parse_container.modules.insert(ParseContainer::get_bean_definition_key_item_mod(&item_found).unwrap(), item_found.clone());
    }

    fn parse_sub_module<
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
    >(
        program_src: &PathBuf,
        parse_container: &mut ParseContainer,
        mut path_depth: &mut Vec<String>,
        module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT,
            ItemModifierT,
            ParseContainerModifierT,
            BuildParseContainerT,
            ParseContainerFinalizerT
        >,
        item_mod: &mut ItemMod
    ) {
        let next_mod = ParseContainer::get_bean_definition_key_item_mod(&item_mod).unwrap();
        parse_container.modules.remove(&next_mod)
            .as_mut()
            .or(Some(item_mod))
            .map(|item_mod| {
                info!("Parsing module in item_mod: {:?}", SynHelper::get_str(&item_mod.ident));
                let mut next_path_deps = path_depth.clone();
                next_path_deps.push(item_mod.ident.to_string().clone());
                if !is_in_line_module(item_mod) {
                    Self::parse_inline_module(program_src, parse_container, module_parser, item_mod, &mut next_path_deps)
                } else {
                    do_parse_container_item_mod(program_src, module_parser, parse_container, item_mod, &next_path_deps);
                    parse_container.modules.insert(ParseContainer::get_bean_definition_key_item_mod(&item_mod).unwrap(), item_mod.clone());
                    None::<bool>
                }
            });
    }

    fn parse_inline_module<
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
    >(
        program_src: &PathBuf,
        parse_container: &mut ParseContainer,
        module_parser: &mut ModuleParser<
            ParseContainerItemUpdaterT,
            ItemModifierT,
            ParseContainerModifierT,
            BuildParseContainerT,
            ParseContainerFinalizerT
        >,
        item_mod: &mut ItemMod,
        next_path_deps: &mut Vec<String>
    ) -> Option<bool> {
        ModuleIterator::retrieve_next_mod(item_mod.clone(), program_src).as_mut()
            .flat_map_opt(|item_mod| {
                do_parse_container_item_mod(program_src, module_parser, parse_container, item_mod, &next_path_deps);
                parse_container.modules.insert(ParseContainer::get_bean_definition_key_item_mod(&item_mod).unwrap(), item_mod.clone());
                None::<bool>
            })
    }
}
