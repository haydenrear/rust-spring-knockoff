extern crate proc_macro;

use proc_macro::{Span, TokenStream};
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::Arc;
use syn::{Attribute, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, ImplItem, ImplItemMethod, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Path, QSelf, TraitItem, Type, TypePath};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    Ident,
    LitStr,
    Token,
    token::Paren,
};
use quote::{format_ident, IdentFragment, quote, quote_token, TokenStreamExt, ToTokens};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};

#[proc_macro_attribute]
pub fn authentication_type(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn configuration(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn field_aug(attr: TokenStream, ts: TokenStream) -> TokenStream {
    ts.into()
}

#[proc_macro_attribute]
pub fn auth_type_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn auth_type_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn auth_type_aware(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn singleton(attr: TokenStream, input: TokenStream) -> TokenStream {
    strip_autowired(input)
}

#[proc_macro_attribute]
pub fn initializer(attr: TokenStream, ts: TokenStream) -> TokenStream {
    ts.into()
}

#[proc_macro_attribute]
pub fn prototype(attr: TokenStream, input: TokenStream) -> TokenStream {
    strip_autowired(input)
}

#[proc_macro_attribute]
pub fn bean(attr: TokenStream, input: TokenStream) -> TokenStream {
    strip_autowired(input)
}

#[proc_macro_attribute]
pub fn autowired(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn mutable_bean(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn mutable_field(attr: TokenStream, input: TokenStream) -> TokenStream {
    strip_autowired(input)
}

#[proc_macro_attribute]
pub fn aspect(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn ordered(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}


fn strip_autowired(input: TokenStream) -> TokenStream {
    if input.to_string().as_str().contains("struct") {
        let mut found: ItemStruct = parse_macro_input!(input as ItemStruct);
        found.fields.iter_mut().for_each(|f| {
            f.attrs.clear();
        });
        return found.to_token_stream().into();
    }
    input.into()
}
