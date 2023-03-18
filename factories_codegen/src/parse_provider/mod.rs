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

/// Basic idea is to provide the user with the parsed ProfileTree and then have them generate tokens
/// based on it. So this will be used in the codegen as a TokenStreamGenerator. It is an extension point
/// for the framework, to enable decoupling the web framework from the dependency injection.
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

            impl #provider_ident {

                pub fn parse_update(items: &mut Item) {
                    #builder_path::parse_update(items);
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

            impl DelegatingParseProvider {

                pub fn parse_update(items: &mut Item) {
                    #(
                        #provider_type::parse_update(parse_container, items);
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
        }.into();
        imports
    }
}

