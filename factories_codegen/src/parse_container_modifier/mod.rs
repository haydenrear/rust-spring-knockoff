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
use crate::token_provider::TokenProvider;

pub struct ParseContainerModifierProvider;

impl DelegatingProvider for ParseContainerModifierProvider {
    fn tokens() -> TokenStream {
        ParseContainerModifierProvider::get_tokens(&ParseContainerModifierProvider::create_provider())
    }

    fn deps() -> Vec<ProviderItem> {
        ParseContainerModifierProvider::create_provider().providers.iter()
            .map(|p| p.clone())
            .collect::<Vec<ProviderItem>>()
    }
}

/// This runs after all of the modules have been parsed.
impl ParseContainerModifierProvider {

    pub fn create_provider() -> Provider {
        FactoriesParser::parse_factories("parse_container_modifier_provider")
    }

    pub fn create_token_provider(provider_item: &ProviderItem) -> TokenStream {

        let provider_ident = &provider_item.provider_ident;
        let builder_path = &provider_item.provider_path;

        let ts = quote! {

            use #builder_path;

            pub struct #provider_ident {
                parse_container_modifier_delegate: #builder_path
            }

            impl #provider_ident {
                pub fn do_modify(items: &mut ParseContainer) {
                    self.parse_container_modifier_delegate.do_modify(items);
                }
            }

        };

        ts.into()
    }

    pub fn get_tokens(provider: &Provider) -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        provider.providers.iter()
            .for_each(|p| ts.append_all(Self::create_token_provider(p)));
        ts.append_all(Self::get_delegating_token_provider(provider));
        ts
    }

    pub fn get_delegating_token_provider(provider: &Provider) -> TokenStream {

        let provider_type = provider.providers.iter()
            .map(|p| p.provider_ident.clone())
            .collect::<Vec<Ident>>();

        quote! {

            pub struct DelegatingParseContainerModifierProvider {
            }

            impl DelegatingParseContainerModifierProvider {

                pub fn new() -> Self {
                    Self {}
                }

                pub fn do_modify(items: &mut ParseContainer) {
                    #(
                        #provider_type::do_modify(items);
                    )*
                }

            }

        }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
        }.into();
        imports
    }
}

