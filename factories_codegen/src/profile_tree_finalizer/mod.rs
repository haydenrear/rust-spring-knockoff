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

pub struct ProfileTreeFinalizerProvider;

/**
Generate the ProfileTree based on BeanDefinitions added in the ParseContainerModifier.
*/
impl ProviderProvider for ProfileTreeFinalizerProvider {
    fn create_delegating_token_provider_tokens(
        provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
        path: &Vec<Path>
    ) -> TokenStream {
        quote! {

            pub struct DelegatingProfileTreeFinalizerProvider {
            }

            impl ProfileTreeFinalizer for DelegatingProfileTreeFinalizerProvider {

                fn finalize(parse_container: &mut ParseContainer) {
                    #(
                        #provider_type::finalize(parse_container);
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

            impl ProfileTreeFinalizer for #provider_ident {
                fn finalize(parse_container: &mut ParseContainer) {
                    #builder_path::finalize(parse_container);
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

