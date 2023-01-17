extern crate proc_macro;

use delegator_macro_rules::{add, types};
use lazy_static::lazy_static;
use proc_macro::{Span, TokenStream};
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
use rust_spring_macro::module_post_processor::{ModuleFieldPostProcessor, ModuleStructPostProcessor};
use syn::{
    LitStr,
    Token,
    Ident,
    token::Paren,
};
use quote::{quote, format_ident, IdentFragment, ToTokens, quote_token, TokenStreamExt};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use module_macro_lib::module_macro_lib::module_parser::parse_module;
use module_macro_lib::module_macro_lib::spring_knockoff_context::ApplicationContextGenerator;

#[proc_macro_attribute]
pub fn module_attr(attr: TokenStream, input: TokenStream) -> TokenStream {

    let mut token_stream_builder = TokenStreamBuilder::default();
    let input_found = input.clone();
    token_stream_builder.add_to_tokens(
        write_starting_types()
    );

    let mut found: Item = parse_macro_input!(input_found as Item);
    let additional = parse_module(found);

    token_stream_builder.add_to_tokens(additional.into());
    token_stream_builder.build()
}

#[proc_macro_attribute]
pub fn singleton(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn prototype(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn bean(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn autowired(attr: TokenStream, input: TokenStream) -> TokenStream {
    input.into()
}

#[proc_macro_attribute]
pub fn Component(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut found: ItemStruct = parse_macro_input!(input as ItemStruct);
    found.fields.iter_mut().for_each(|f| {
        f.attrs.clear();
    });
    found.to_token_stream().into()
}

#[derive(Default)]
struct TokenStreamBuilder {
    stream_build: Vec<TokenStream>
}

impl TokenStreamBuilder {

    fn add_to_tokens(&mut self, tokens: TokenStream) {
        self.stream_build.push(tokens);
    }

    fn build(&self) -> TokenStream {
        let mut final_tokens = TokenStream::default();
        self.stream_build.iter().for_each(|s| final_tokens.extend(s.clone().into_iter()));
        final_tokens
    }

}

fn write_starting_types() -> TokenStream {
    let mut tokens = TokenStreamBuilder::default();
    tokens.add_to_tokens(ApplicationContextGenerator::create_application_context().into());
    tokens.add_to_tokens(ApplicationContextGenerator::create_bean_factory().into());
    tokens.build()
}




