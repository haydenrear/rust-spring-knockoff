use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use rand::Rng;
use serde::{Deserialize, Serialize};
use syn::{Attribute, Item, ItemMod, parse2, parse_str, Path};
use syn::punctuated::Pair::Punctuated;
use toml::{Table, Value};
use crate::factories_parser::{FactoriesParser, Provider};
use crate::provider::{DelegatingProvider, ProviderProvider};

pub struct FactoryBootTokenProvider;

/// Can refactor to pull out common with framework_token_provider
/// -- this is the same, except FactoryBootTokenProvider is generating tokens for the
///     booting process for codegen -
/// example:
///     codegen tokens for HandlerMapping is framework_token_provider
///     codegen tokens for booting up HandlerMapping and dispatcher at start is FactoryBootTokenProvider
impl ProviderProvider for FactoryBootTokenProvider {
    fn create_delegating_token_provider_tokens(provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
                                               path: &Vec<Path>) -> TokenStream {
        quote! {

            pub struct DelegatingFactoryBootTokenProvider {
                #(#provider_idents: #provider_type,)*
            }

            impl ProfileTreeFactoryBootTokenProvider for DelegatingFactoryBootTokenProvider {
                fn new(profile_tree: &mut ProfileTree) -> Self {
                    #(
                        let #provider_idents = #provider_type::new(profile_tree);
                    )*
                    Self {
                        #(#provider_idents,)*
                    }
                }

                fn generate_token_stream(&self) -> TokenStream {
                    let mut ts = TokenStream::default();
                    #(
                        ts.append_all(self.#provider_idents.generate_token_stream());
                    )*
                    ts
                }

            }

        }
    }

    fn create_token_provider_tokens<T: ToTokens>(use_statement: T, builder_path: syn::Path, provider_ident: Ident) -> TokenStream {
        quote! {

                #use_statement

                pub struct #provider_ident {
                    profile_tree: ProfileTree,
                    token_delegate: #builder_path
                }

                impl ProfileTreeFactoryBootTokenProvider for #provider_ident {
                    fn new(items: &mut ProfileTree) -> #provider_ident {
                        let profile_tree = items.clone();
                        let token_delegate = #builder_path::new(items);
                        Self {
                            profile_tree,
                            token_delegate
                        }
                    }
                    fn generate_token_stream(&self) -> TokenStream {
                        self.token_delegate.generate_token_stream()
                    }
                }

        }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            // use proc_macro2::TokenStream;
            // use quote::TokenStreamExt;
            // use module_macro_shared::*;
        }.into();
        imports
    }
}

