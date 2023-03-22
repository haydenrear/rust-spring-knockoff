use syn::{Item, ItemMod};
use crate::module_macro_lib::item_parser::item_enum_parser::ItemEnumParser;
use crate::module_macro_lib::item_parser::item_fn_parser::ItemFnParser;
use crate::module_macro_lib::item_parser::item_impl_parser::ItemImplParser;
use crate::module_macro_lib::item_parser::item_struct_parser::ItemStructParser;
use crate::module_macro_lib::item_parser::item_trait_parser::ItemTraitParser;
use crate::module_macro_lib::item_parser::ItemParser;
use module_macro_shared::parse_container::ParseContainer;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use knockoff_providers_gen::DelegatingParseProvider;

use quote::ToTokens;
use module_macro_shared::parse_container::parse_container_modifier::ParseContainerItemUpdater;

pub struct ItemModParser;

impl ItemParser<ItemMod> for ItemModParser {
    fn parse_item(parse_container: &mut ParseContainer, item_found: &mut ItemMod, mut path_depth: Vec<String>) {
        path_depth.push(item_found.ident.to_string().clone());
        item_found.content.iter_mut()
            .flat_map(|mut c| c.1.iter_mut())
            .for_each(|i: &mut Item| Self::parse_item_inner(i, parse_container, &mut path_depth.clone()));
    }
}

impl ItemModParser {
    pub fn parse_item_inner(i: &mut Item, mut app_container: &mut ParseContainer, path_depth: &mut Vec<String>) {
        DelegatingParseProvider::parse_update(i, app_container);
        match i {
            Item::Const(const_val) => {
                log_message!("Found const val {}.", const_val.to_token_stream().clone());
            }
            Item::Enum(enum_type) => {
                log_message!("Found enum val {}.", enum_type.to_token_stream().clone());
                ItemEnumParser::parse_item(app_container, enum_type, path_depth.clone());
            }
            Item::Fn(fn_type) => {
                log_message!("Found fn type {}.", fn_type.to_token_stream().clone());
                ItemFnParser::parse_item(app_container, fn_type, path_depth.clone());
            }
            Item::ForeignMod(_) => {}
            Item::Impl(ref mut impl_found) => {
                ItemImplParser::parse_item(app_container, impl_found, path_depth.clone());
            }
            Item::Macro(macro_created) => {
            }
            Item::Macro2(_) => {}
            Item::Mod(ref mut module) => {
                log_message!("Found module with name {} !!!", module.ident.to_string().clone());
                ItemModParser::parse_item(app_container, module, path_depth.clone());
            }
            Item::Static(static_val) => {
                log_message!("Found static val {}.", static_val.to_token_stream().clone());
            }
            Item::Struct(ref mut item_struct) => {
                ItemStructParser::parse_item(app_container, item_struct, path_depth.clone());
            }
            Item::Trait(trait_created) => {
                log_message!("Trait created: {}", trait_created.ident.clone().to_string());
                ItemTraitParser::parse_item(app_container, trait_created, path_depth.clone());
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
