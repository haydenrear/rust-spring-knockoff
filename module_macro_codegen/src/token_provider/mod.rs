use std::any::Any;
use std::collections::LinkedList;
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::os::unix::raw::time_t;
use std::path::Path;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use syn::{Attribute, Item, ItemMod, parse2, parse_str};
use syn::punctuated::Pair::Punctuated;
use toml::{Table, Value};
use factories_parser::FactoriesParser;
use knockoff_logging::{initialize_log, use_logging};
use crate::codegen_items;
use crate::parser::{CodegenItem, CodegenItems, CodegenItemType, get_codegen_item, LibParser};

use_logging!();
initialize_log!();

use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

pub mod factories_parser;

#[derive(Clone)]
pub struct TokenProvider {
    providers: Vec<TokenProviderItem>,
}

#[derive(Clone)]
pub struct TokenProviderItem {
    name: String,
    provider_path: syn::Path,
    provider_ident: Ident
}

impl Default for TokenProvider {
    fn default() -> Self {
        FactoriesParser::parse_factories()
    }
}

/// Basic idea is to provide the user with the parsed ProfileTree and then have them generate tokens
/// based on it. So this will be used in the codegen as a TokenStreamGenerator. It is an extension point
/// for the framework, to enable decoupling the web framework from the dependency injection.
impl TokenProvider {

    pub(crate) fn new(item: &Vec<Item>) -> Option<Self> {
        None
    }

    pub(crate) fn new_dyn_codegen(item: &Vec<Item>) -> Option<CodegenItemType> {
        Self::new(item)
            .map(|i| CodegenItemType::TokenProvider(i))
    }

    pub(crate) fn get_codegen_items(tokens: &Vec<Item>) -> Vec<Item> {
        if tokens.len() == 0 {
            return vec![];
        }

        let codegen = tokens.iter().flat_map(|tokens| {
            match tokens {
                Item::Mod(module_to_parse) => {
                    if Self::mod_attr_has_supports(&module_to_parse.attrs) {
                        return vec![tokens.clone()];
                    }
                    vec![]
                }
                _ => {
                    vec![]
                }
            }
        }).collect::<Vec<Item>>();

        codegen
    }

    fn mod_attr_has_supports(vec: &Vec<Attribute>) -> bool {
        vec.iter()
            .any(|attr| attr.path.to_token_stream()
                .to_string().as_str()
                .contains("token_provider")
            )
    }
}

impl CodegenItem for TokenProvider {

    fn supports_item(item: &Vec<Item>) -> bool {
        false
    }

    fn supports(&self, item: &Vec<Item>) -> bool {
        Self::supports_item(item)
    }

    fn get_codegen(&self) -> Option<String> {
        if self.providers.len() <= 0 {
            return None;
        }
        // self.providers.iter().map(|c| {
        //     match c {
        //         Item::Mod(item_mod) => {
        //
        //         }
        //         _ => {
        //
        //         }
        //     }
        // });
        Some(
            quote! {

                // pub struct HandlerMappingTokenStreamProvider;
                //
                // impl TokenStreamProvider for DelegatingTokenProvider {
                //
                //     fn new(items: &ProfileTree) -> Self {
                //         todo!()
                //     }
                //
                //     fn get_token_stream(&self) -> TokenStream {
                //         todo!()
                //     }
                // }
            }.to_string()
        )
    }


    fn default_codegen(&self) -> String {
        let ts = quote!{
        };
        ts.to_string()
    }

    fn get_unique_id(&self) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric{})
            .take(10)
            .map(char::from)
            .collect()
    }

    fn get_unique_ids(&self) -> Vec<String> {
        vec![self.get_unique_id()]
    }
}
