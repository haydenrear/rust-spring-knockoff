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
use crate::factories_parser::{FactoriesParser, Provider};
use crate::provider::{DelegatingProvider, ProviderProvider};

pub struct ProfileTreeModifierProvider;

/**
Generate the ProfileTree based on BeanDefinitions added in the ParseContainerModifier.
*/
impl ProviderProvider for ProfileTreeModifierProvider {
    fn create_delegating_token_provider_tokens(provider_type: Vec<Ident>, provider_idents: Vec<Ident>) -> TokenStream {
        quote! {

            pub struct DelegatingProfileTreeModifierProvider {
                #(#provider_idents: #provider_type,)*
            }

            impl ProfileTreeModifier for DelegatingProfileTreeModifierProvider {

                fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
                    #(
                        #provider_type::modify_bean(dep_type, profile_tree);
                    )*
                }

                fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self {
                    #(
                        let #provider_idents = #provider_type::new(dep_type, profile_tree);
                    )*
                    Self {
                        #(#provider_idents)*
                    }
                }

            }

        }
    }

    fn create_token_provider_tokens(builder_path: syn::Path, provider_ident: Ident) -> TokenStream {
        quote! {

            use #builder_path;

            pub struct #provider_ident {
            }

            impl #provider_ident {
                fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
                    #provider_ident::modify_bean(dep_type, profile_tree);
                }

                fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> #provider_ident {
                    #provider_ident::new(profile_tree_items)
                }
            }

        }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            use module_macro_shared::bean::BeanDefinition;
            use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
            use std::collections::HashMap;
        }.into();
        imports
    }
}

