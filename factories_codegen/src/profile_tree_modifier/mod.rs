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

pub struct ProfileTreeModifierProvider;

impl DelegatingProvider for ProfileTreeModifierProvider {
    fn tokens() -> TokenStream {
        ProfileTreeModifierProvider::get_tokens(&ProfileTreeModifierProvider::create_provider())
    }

    fn deps() -> Vec<ProviderItem> {
        ProfileTreeModifierProvider::create_provider().providers.iter()
            .map(|p| p.clone())
            .collect::<Vec<ProviderItem>>()
    }
}

/// Allows user to generate token based on parsed ProfileTree.
impl ProfileTreeModifierProvider {

    pub fn create_provider() -> Provider {
        FactoriesParser::parse_factories("profile_tree_modifier_provider")
    }

    pub fn create_token_provider(provider_item: &ProviderItem) -> TokenStream {

        let provider_ident = &provider_item.provider_ident;
        let builder_path = &provider_item.provider_path;

        let ts = quote! {

            use #builder_path;

            pub struct #provider_ident {
                profile_tree: ProfileTree,
                token_delegate: #builder_path
            }

            impl #provider_ident {
                pub fn do_modify(items: &mut ProfileTree) {
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

    pub fn get_tokens(provider: &Provider) -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        provider.providers.iter()
            .for_each(|p| ts.append_all(Self::create_token_provider(p)));
        ts.append_all(Self::get_delegating_token_provider(provider));
        ts
    }

    pub fn get_delegating_token_provider(provider: &Provider) -> TokenStream {

        let provider_idents = provider.providers.iter()
            .map(|p| Ident::new(p.provider_ident.to_string().to_lowercase().as_str(), Span::call_site()))
            .collect::<Vec<Ident>>();
        let provider_type = provider.providers.iter()
            .map(|p| p.provider_ident.clone())
            .collect::<Vec<Ident>>();

        quote! {

            pub struct DelegatingProfileTreeModifierProvider {
                #(#provider_idents: #provider_type,)*
            }

            impl DelegatingProfileTreeModifierProvider {
                pub fn new(profile_tree: &ProfileTree) -> Self {
                    #(
                        let #provider_idents = #provider_type::new(profile_tree);
                    )*
                    Self {
                        #(#provider_idents,)*
                    }
                }

                pub fn generate_token_stream(&self) -> TokenStream {
                    let mut ts = TokenStream::default();
                    #(
                        ts.append_all(self.#provider_idents.generate_token_stream());
                    )*
                    ts
                }

            }

        }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            use module_macro_shared::profile_tree::ProfileTree;
            use proc_macro2::TokenStream;
            use quote::TokenStreamExt;
        }.into();
        imports
    }
}

