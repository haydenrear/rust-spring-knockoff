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
use crate::token_provider::TokenProvider;

pub struct ParseContainerModifierProvider;

/// This runs after all of the modules have been parsed.
impl ProviderProvider for ParseContainerModifierProvider {
    fn create_delegating_token_provider_tokens(
        provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
        path: &Vec<Path>
    ) -> TokenStream {
        quote! {

            pub struct DelegatingParseContainerModifierProvider {
            }

            impl DelegatingParseContainerModifierProvider {

                pub fn new() -> Self {
                    Self {}
                }

            }

            impl ParseContainerModifier for DelegatingParseContainerModifierProvider {

                fn do_modify(items: &mut ParseContainer) {
                    #(
                        #provider_type::do_modify(items);
                    )*
                }

            }

        }
    }

    fn create_token_provider_tokens<T: ToTokens>(use_statement: T, path: syn::Path, provider_ident: Ident) -> TokenStream {
        quote! {

                #use_statement

                pub struct #provider_ident {
                }

                impl ParseContainerModifier for #provider_ident {
                    fn do_modify(items: &mut ParseContainer) {
                        #path::do_modify(items);
                    }
                }

        }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            use module_macro_shared::*;
        }.into();
        imports
    }
}

