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

pub struct TokenProvider;

/// Basic idea is to provide the user with the parsed ProfileTree and then have them generate tokens
/// based on it. So this will be used in the codegen as a TokenStreamGenerator. It is an extension point
/// for the framework, to enable decoupling the web framework from the dependency injection.
impl ProviderProvider for TokenProvider {
    fn create_delegating_token_provider_tokens(provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
                                               path: &Vec<Path>) -> TokenStream {
        quote! {

            pub struct DelegatingTokenProvider {
                #(#provider_idents: #provider_type,)*
            }

            impl ProfileTreeTokenProvider for DelegatingTokenProvider {
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

                impl ProfileTreeTokenProvider for #provider_ident {
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
            use proc_macro2::TokenStream;
            use quote::TokenStreamExt;
            use module_macro_shared::*;
        }.into();
        imports
    }
}

