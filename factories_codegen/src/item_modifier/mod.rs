use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Path;
use knockoff_logging::*;
use crate::provider::ProviderProvider;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;

import_logger!("item_modifier.rs");

pub struct ItemModifierProvider;

impl ProviderProvider for  ItemModifierProvider {
    fn create_delegating_token_provider_tokens(
        provider_type: Vec<Ident>,
        provider_idents: Vec<Ident>,
        path: &Vec<Path>
    ) -> TokenStream {
        log_message!("Creating delegating provider tokens.");
        quote! {
            #[derive(Default)]
            pub struct DelegatingItemModifier;

            impl ItemModifier for DelegatingItemModifier {

                fn supports_item(item: &Item) -> bool {
                    #(
                        if #provider_type::supports_item(item) {
                            return true;
                        }
                    )*
                    false
                }

                fn new() -> Self {
                    Self {}
                }

                fn modify_item(parse_container: &mut ParseContainer,
                               item: &mut Item, path_depth: Vec<String>) {
                    let mut path_depth = path_depth.clone();
                    #(
                        if #provider_type::supports_item(item) {
                            #provider_type::modify_item(parse_container, item, path_depth.clone());
                        }
                    )*
                    match item {
                        Item::Mod(ref mut item_mod) => {
                            let mod_ident = item_mod.ident.to_string().clone();
                            if !path_depth.contains(&mod_ident) {
                                path_depth.push(mod_ident);
                            }
                            item_mod.content.iter_mut().for_each(|c| {
                                for item in c.1.iter_mut() {
                                    Self::modify_item(parse_container, item, path_depth.clone())
                                }
                            });
                        }
                        _ => {}
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

            impl ItemModifier for #provider_ident {
                fn modify_item(parse_container: &mut ParseContainer,
                               item: &mut Item, path_depth: Vec<String>) {
                    #builder_path::modify_item(parse_container, item, path_depth);
                }

                fn supports_item(item: &Item) -> bool {
                    #builder_path::supports_item(item)
                }

                fn new() -> Self {
                    Self {}
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
