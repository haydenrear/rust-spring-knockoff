use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use rand::Rng;
use serde::{Deserialize, Serialize};
use syn::{Attribute, Item, ItemMod, parse2, parse_str};
use syn::punctuated::Pair::Punctuated;
use toml::{Table, Value};
use crate::factories_parser::{Dependency, FactoriesParser};

#[derive(Clone)]
pub struct TokenProvider {
    pub providers: Vec<TokenProviderItem>,
}

#[derive(Clone)]
pub struct TokenProviderItem {
    pub name: String,
    pub provider_path: syn::Path,
    pub provider_ident: Ident,
    pub path: Option<String>,
    pub dep_name: String,
    pub version: Option<String>,
    pub deps: Vec<Dependency>
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

    pub fn create_token_provider(provider_item: &TokenProviderItem) -> TokenStream {

        let provider_ident = &provider_item.provider_ident;
        let builder_path = &provider_item.provider_path;

        let ts = quote! {

            use #builder_path;

            pub struct #provider_ident {
                profile_tree: ProfileTree,
                token_delegate: #builder_path
            }

            impl #provider_ident {
                pub fn new(items: &ProfileTree) -> #provider_ident {
                    let profile_tree = items.clone();
                    let token_delegate = #builder_path::new(items);
                    Self {
                        profile_tree,
                        token_delegate
                    }
                }
                pub fn generate_token_stream(&self) -> TokenStream {
                    self.token_delegate.generate_token_stream()
                }
            }

        };

        ts.into()
    }

    pub fn get_tokens(&self) -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        self.providers.iter()
            .for_each(|p|
                ts.append_all(
                    Self::create_token_provider(p)
                )
            );
        ts
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            use module_macro_shared::profile_tree::ProfileTree;
            use proc_macro2::TokenStream;
        }.into();
        imports
    }
}

