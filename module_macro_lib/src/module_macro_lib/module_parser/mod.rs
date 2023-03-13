use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::{Arc, Mutex};
use proc_macro2::TokenStream;
use syn::{Attribute, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, ImplItem, ImplItemMethod, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Path, QSelf, TraitItem, Type, TypePath};
use syn::__private::{str, TokenStream2};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    Ident,
    LitStr,
    Token,
    token::Paren,
};
use quote::{format_ident, IdentFragment, quote, quote_token, ToTokens};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::parse_container::ParseContainer;
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
use crate::FieldAugmenterImpl;
use crate::module_macro_lib::initializer::ModuleMacroInitializer;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::AspectParser;
use module_macro_codegen::module_extractor::ModuleParser;
use module_macro_codegen::parser::LibParser;
use web_framework_shared::matcher::Matcher;
use crate::module_macro_lib::item_modifier::delegating_modifier::DelegatingItemModifier;
use crate::module_macro_lib::item_modifier::ItemModifier;
use crate::module_macro_lib::item_parser::item_enum_parser::ItemEnumParser;
use crate::module_macro_lib::item_parser::item_fn_parser::ItemFnParser;
use crate::module_macro_lib::item_parser::item_mod_parser::ItemModParser;
use crate::module_macro_lib::item_parser::item_struct_parser::ItemStructParser;
use crate::module_macro_lib::item_parser::item_trait_parser::ItemTraitParser;
use crate::module_macro_lib::item_parser::ItemParser;

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub fn parse_module(mut found: Item) -> TokenStream {
    create_initial_parse_container(&mut found).as_mut()
        .map(|created| {
            let container = do_container_modifications(&mut found, created);

            let container_tokens = container.build_to_token_stream();

            return quote!(
                #container_tokens
                #found
            ).into();
        })
        .or(Some(quote!(#found).into()))
        .unwrap()

}

pub(crate) fn do_container_modifications<'a>(mut found: &'a mut Item, created: &'a mut (ParseContainer, String)) -> &'a mut ParseContainer {
    let item_modifier = DelegatingItemModifier::new();
    let container = &mut created.0;
    item_modifier.modify_item(container, &mut found, vec![created.1.clone()]);
    container
}

pub(crate) fn create_initial_parse_container(mut found: &mut Item) -> Option<(ParseContainer, String)> {
    let mut created = match &mut found {
        Item::Mod(ref mut module_found) => {
            let mut container = ParseContainer::default();
            container.aspects = AspectParser::parse_aspects();

            ItemModParser::parse_item(
                &mut container,
                module_found,
                vec![module_found.ident.to_string().clone()]
            );

            log_message!("{} is module.", module_found.ident.to_string().as_str());
            log_message!("{:?} is module.", &container);

            Some((container, module_found.ident.to_string().clone()))
        }
        _ => {
            None
        }
    };
    created
}

