extern crate proc_macro;

use proc_macro::{Span, TokenStream};
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::Arc;
use syn::{Attribute, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, FnArg, ImplItem, ImplItemMethod, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Path, QSelf, TraitItem, Type, TypePath};
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
pub fn enable_http_security(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn mutable_bean(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn rest_controller(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn request_mapping(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn get_mapping(attr: TokenStream, input: TokenStream) -> TokenStream {
    strip_method_arg_attr(input)
}

#[proc_macro_attribute]
pub fn post_mapping(attr: TokenStream, input: TokenStream) -> TokenStream {
    strip_method_arg_attr(input)
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

#[proc_macro_attribute]
pub fn request_body(attr: TokenStream, input: TokenStream) -> TokenStream {
    strip_method_arg_attr(input)
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

fn strip_method_arg_attr(input: TokenStream) -> TokenStream {
    let mut found: ImplItemMethod = parse_macro_input!(input as ImplItemMethod);
    found.sig.inputs.iter_mut().for_each(|f| {
        match f {
            FnArg::Receiver(r) => {
                r.attrs.clear()
            }
            FnArg::Typed(t) => {
                println!("Stripping from {}.", t.attrs.len());
                t.attrs.clear()
            }
        }
    });
    found.to_token_stream().into()
}
