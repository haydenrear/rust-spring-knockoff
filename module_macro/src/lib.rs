extern crate proc_macro;

use proc_macro::{Span, TokenStream};
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::Arc;
use syn::{Attribute, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, ImplItem, ImplItemMethod, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Path, QSelf, TraitItem, Type, TypePath};
use syn::__private::str;
use syn::parse::Parser;
use syn::spanned::Spanned;
use rust_spring_macro::module_post_processor::{ModuleFieldPostProcessor, ModuleStructPostProcessor};
use syn::{
    Ident,
    LitStr,
    Token,
    token::Paren,
};
use quote::{format_ident, IdentFragment, quote, quote_token, TokenStreamExt, ToTokens};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use module_macro_lib::module_macro_lib::module_parser::parse_module;
use module_macro_lib::module_macro_lib::knockoff_context_builder::ApplicationContextGenerator;
use module_macro_lib::FieldAugmenterImpl;
use module_macro_lib::module_macro_lib::initializer::Initializer;
use module_macro_shared::module_macro_shared_codegen::ContextInitializer;

#[proc_macro_attribute]
pub fn module_attr(attr: TokenStream, input: TokenStream) -> TokenStream {

    let mut token_stream_builder = TokenStreamBuilder::default();
    let input_found = input.clone();
    token_stream_builder.add_to_tokens(
        write_starting_types()
    );

    let mut found: Item = parse_macro_input!(input_found as Item);

    let ts = TokenStream::default();
    let field_augmenter: FieldAugmenterImpl = parse_macro_input!(ts as FieldAugmenterImpl);

    let init = Initializer {
        field_augmenter
    };

    let additional = parse_module(found, init);

    token_stream_builder.add_to_tokens(additional.into());
    token_stream_builder.build()
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




