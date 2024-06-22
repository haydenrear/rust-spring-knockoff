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

pub struct MutableMacroModifierProvider;

impl ProviderProvider for MutableMacroModifierProvider {
    fn create_delegating_token_provider_tokens(provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
                                               path: &Vec<Path>) -> TokenStream {
        quote! {

            pub struct DelegatingMutableMacroModifierProvider {
            }

            impl MutableModuleModifier for DelegatingMutableMacroModifierProvider {
                fn matches(item: &mut Item) -> bool {
                    #(
                        if #provider_type::matches(item) {
                            return true;
                        }
                    )*

                    false
                }

                fn do_provide(item: &mut Item) -> Option<TokenStream> {
                    let mut do_provider = TokenStream::default();
                    let mut did_update = false;
                    #(
                        if #provider_type::matches(item) {
                            do_provider.extend(#provider_type::do_provide(item));
                            did_update = true;
                        }
                    )*

                    if did_update {
                        Some(do_provider)
                    } else {
                        None
                    }
                }
            }

        }
    }

    fn create_token_provider_tokens<T: ToTokens>(use_statement: T, builder_path: syn::Path, provider_ident: Ident) -> TokenStream {
        quote! {

                #use_statement

                pub struct #provider_ident {
                }

                impl MutableModuleModifier for #provider_ident {
                    fn matches(item: &mut Item) -> bool {
                        #builder_path::matches(item)
                    }

                    fn do_provide(item: &mut Item) -> Option<TokenStream> {
                        #builder_path::do_provide(item)
                    }
                }

        }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            use dfactory_dcodegen_shared::*;
            use quote::quote;
        }.into();
        imports
    }
}

