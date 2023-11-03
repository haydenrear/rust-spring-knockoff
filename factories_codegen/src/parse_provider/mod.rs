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

pub struct ParseProvider;

/// This is called on each item as it is parsed.
impl ProviderProvider for ParseProvider {
    fn create_delegating_token_provider_tokens(provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
                                               path: &Vec<Path>) -> TokenStream {
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

    fn create_token_provider_tokens<T: ToTokens>(use_statement: T, builder_path: syn::Path, provider_ident: Ident) -> TokenStream {
        quote! {
                #use_statement

                pub struct #provider_ident {
                }

                impl ParseContainerItemUpdater for #provider_ident {

                    fn parse_update(items: &mut Item, parse_container: &mut ParseContainer) {
                        #builder_path::parse_update(items, parse_container);
                    }
                }

            }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            use syn::Item;
            use module_macro_shared::*;
        }.into();
        imports
    }
}

