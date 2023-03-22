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
use crate::provider::{DelegatingProvider, Provider, ProviderItem};

pub struct ParseProvider;

impl DelegatingProvider for ParseProvider {
    fn tokens() -> TokenStream {
        ParseProvider::get_tokens(&ParseProvider::create_provider())
    }

    fn deps() -> Vec<ProviderItem> {
        ParseProvider::create_provider().providers.iter()
            .map(|p| p.clone())
            .collect::<Vec<ProviderItem>>()
    }
}

/// This is called on each item as it is parsed.
impl ParseProvider {

    pub fn create_provider() -> Provider {
        FactoriesParser::parse_factories("parse_provider")
    }

    pub fn create_token_provider(provider_item: &ProviderItem) -> TokenStream {

        let provider_ident = &provider_item.provider_ident;
        let builder_path = &provider_item.provider_path;

        let ts = quote! {

            use #builder_path;

            pub struct #provider_ident {
            }

            impl ParseContainerItemUpdater for #provider_ident {

                fn parse_update(items: &mut Item, parse_container: &mut ParseContainer) {
                    #builder_path::parse_update(items, parse_container);
                }
            }

        };

        ts.into()
    }

    pub fn get_delegating_parse_provider(provider: &Provider) -> TokenStream {

        let provider_type = provider.providers.iter()
            .map(|p| p.provider_ident.clone())
            .collect::<Vec<Ident>>();

        quote! {

            pub struct DelegatingParseProvider {
            }

            impl ParseContainerItemUpdater for DelegatingParseProvider {

                fn parse_update(items: &mut Item, parse_container: &mut ParseContainer) {
                    #(
                        #provider_type::parse_update(items, parse_container);
                    )*
                }
            }
        }
    }

    pub fn get_tokens(provider: &Provider) -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        provider.providers.iter()
            .for_each(|p| ts.append_all(Self::create_token_provider(p)));
        ts.append_all(Self::get_delegating_parse_provider(provider));
        ts
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            use syn::Item;
            use module_macro_shared::parse_container::ParseContainer;
            use module_macro_shared::parse_container::parse_container_modifier::ParseContainerItemUpdater;
        }.into();
        imports
    }
}

