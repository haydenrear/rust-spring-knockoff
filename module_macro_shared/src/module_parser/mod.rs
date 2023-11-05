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
use crate::{BuildParseContainer,  ItemModifier, ItemParser, ModuleParser, ParseContainer, ParseContainerItemUpdater, ParseContainerModifier, ProfileTreeFinalizer};

use module_macro_codegen::parser::LibParser;


use knockoff_logging::*;
use codegen_utils::project_directory;
use crate::item_mod_parser::ItemModParser;
use crate::logger_lazy;
import_logger!("module_parser.rs");


pub fn parse_module_into_container<
    ParseContainerItemUpdaterT: ParseContainerItemUpdater,
    ItemModifierT: ItemModifier,
    ParseContainerModifierT: ParseContainerModifier,
    BuildParseContainerT: BuildParseContainer,
    ParseContainerFinalizerT: ProfileTreeFinalizer,
>(mut found: &mut Item, module_parser: &mut ModuleParser<
    ParseContainerItemUpdaterT,
    ItemModifierT,
    ParseContainerModifierT,
    BuildParseContainerT,
    ParseContainerFinalizerT
>) -> Option<ParseContainer>
    where
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
{
    create_initial_parse_container(&mut found, module_parser)
        .map(|created| {
            let container = do_container_modifications(&mut found, created, module_parser);
            container
        })
}

pub(crate) fn do_container_modifications<
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
>(mut found: &mut Item, module_parser: &mut ModuleParser<
    ParseContainerItemUpdaterT,
    ItemModifierT,
    ParseContainerModifierT,
    BuildParseContainerT,
    ParseContainerFinalizerT
>) -> Option<(ParseContainer, String)> {
    let mut created = match &mut found {
        Item::Mod(ref mut module_found) => {
            let mut container = ParseContainer::default();

            ItemModParser::parse_item(
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

