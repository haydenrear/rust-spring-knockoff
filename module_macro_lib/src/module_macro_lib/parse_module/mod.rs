use std::env;
use std::path::PathBuf;
use knockoff_providers_gen::{DelegatingItemModifier, DelegatingParseContainerModifierProvider, DelegatingParseProvider, DelegatingProfileTreeFinalizerProvider};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Item;
use codegen_utils::program_src;
use knockoff_logging::info;
use module_macro_shared::{ItemModifier, ModuleParser, parse_module_into_container};
use crate::module_macro_lib::parse_container::ParseContainerBuilder;

pub fn parse_module(mut item: Item, program_src: &PathBuf) -> TokenStream {
    let mut module_parser = ModuleParser {
        delegating_parse_container_updater: DelegatingParseProvider {},
        delegating_parse_container_modifier: DelegatingParseContainerModifierProvider::new(),
        delegating_parse_container_builder: ParseContainerBuilder::new(),
        delegating_parse_container_item_modifier: DelegatingItemModifier::new(),
        delegating_parse_container_finalizer: DelegatingProfileTreeFinalizerProvider {},
    };
    parse_module_into_container(program_src, &mut item, &mut module_parser)
        .map(|mut container| {
            let container_tokens = ParseContainerBuilder::build_to_token_stream(&mut container);

            return quote!(
                #container_tokens
                #item
            ).into();

        })
        .or(Some(quote!(#item).into()))
        .unwrap()
}
