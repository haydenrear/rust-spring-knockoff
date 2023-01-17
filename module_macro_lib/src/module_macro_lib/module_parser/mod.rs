use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf};
use syn::__private::str;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    LitStr,
    Token,
    Ident,
    token::Paren,
};
use quote::{quote, format_ident, IdentFragment, ToTokens, quote_token, TokenStreamExt};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use proc_macro2::TokenStream;
use crate::module_macro_lib::module_container::ModuleContainer;
use crate::module_macro_lib::module_tree::TestFieldAdding;

pub fn parse_module(mut found: Item) -> TokenStream {
    match &mut found {
        Item::Mod(ref mut struct_found) => {
            let mut container = ModuleContainer::default();
            parse_item_recursive(struct_found, &mut container);
            let container_tokens = container.to_token_stream();
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



pub fn parse_item_recursive(item_found: &mut ItemMod, module_container: &mut ModuleContainer) {
    item_found.content.iter_mut()
        .flat_map(|mut c| c.1.iter_mut())
        .for_each(|i: &mut Item| parse_item(i, module_container));
}



pub fn get_trait(item_impl: &mut ItemImpl) -> Option<Path> {
    item_impl.trait_.clone()
        .and_then(|item_impl_found| {
            Some(item_impl_found.1)
        })
        .or_else(|| None)
}



pub fn parse_item(i: &mut Item, mut module_container: &mut ModuleContainer) {
    match i {
        Item::Const(_) => {}
        Item::Enum(_) => {}
        Item::ExternCrate(_) => {}
        Item::Fn(_) => {}
        Item::ForeignMod(_) => {}
        Item::Impl(impl_found) => {
            module_container.create_update_impl(impl_found);
        }
        Item::Macro(macro_created) => {
            // to add behavior to module macro,
            // have another macro impl Parse for a struct that
            // has a vec of Fn, and in the impl Parse
            // the behavior as a function that is added to the struct
            // to be called, and that function is passed as a closure
            // to the macro that creates the impl Parse - this will have to be
            // handled in the build.rs file - to relocate
            // macro_created.mac.parse_body()
        }
        Item::Macro2(_) => {}
        Item::Mod(ref mut module) => {
            println!("Found module with name {} !!!", module.ident.to_string().clone());
            parse_item_recursive(module, module_container);
        }
        Item::Static(_) => {}
        Item::Struct(ref mut item_struct) => {
            let f = TestFieldAdding {};
            f.process(item_struct);
            module_container.add_item_struct(item_struct);
            println!("Found struct with name {} !!!", item_struct.ident.to_string().clone());
        }
        Item::Trait(trait_created) => {
            println!("Trait created: {}", trait_created.ident.clone().to_string());
            module_container.create_update_trait(trait_created);
        }
        Item::TraitAlias(_) => {}
        Item::Type(_) => {
            println!("Item type found!")
        }
        Item::Union(_) => {}
        Item::Use(_) => {}
        Item::Verbatim(_) => {}
        _ => {}
    }
}
