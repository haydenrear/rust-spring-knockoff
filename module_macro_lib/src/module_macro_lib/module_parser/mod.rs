use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::{Arc, Mutex};
use proc_macro2::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf};
use syn::__private::{str, TokenStream2};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    LitStr,
    Token,
    Ident,
    token::Paren,
};
use quote::{quote, format_ident, IdentFragment, ToTokens, quote_token};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use crate::module_macro_lib::parse_container::ParseContainer;
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
use crate::FieldAugmenterImpl;
use crate::module_macro_lib::initializer::ModuleMacroInitializer;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::AspectParser;
use module_macro_codegen::module_extractor::ModuleParser;
use module_macro_codegen::parser::LibParser;
use web_framework_shared::matcher::Matcher;
use crate::module_macro_lib::item_parser::{ItemEnumParser, ItemFnParser, ItemImplParser, ItemModParser, ItemParser, ItemStructParser, ItemTraitParser};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub fn parse_module(mut found: Item) -> TokenStream {
    match &mut found {
        Item::Mod(ref mut module_found) => {
            let mut container = ParseContainer::default();
            container.aspects = AspectParser::parse_aspects();

            ItemModParser::parse_item(
                &mut container,
                module_found,
                vec![module_found.ident.to_string().clone()]
            );

            let container_tokens = container.build_to_token_stream();

            quote!(
                #found
                #container_tokens
            ).into()

        }
        _ => {
            return quote!(#found).into();
        }
    }
}


pub fn get_trait(item_impl: &mut ItemImpl) -> Option<Path> {
    item_impl.trait_.clone()
        .and_then(|item_impl_found| {
            Some(item_impl_found.1)
        })
        .or_else(|| None)
}

pub fn parse_item(i: &mut Item, mut app_container: &mut ParseContainer, path_depth: &mut Vec<String>) {
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
            path_depth.push(module.ident.to_string().clone());
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
