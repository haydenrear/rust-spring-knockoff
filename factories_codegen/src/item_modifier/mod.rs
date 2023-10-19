use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Path;
use knockoff_logging::{initialize_log, use_logging};
use crate::provider::ProviderProvider;

use_logging!();
initialize_log!();

use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

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

    fn create_token_provider_tokens(builder_path: syn::Path, provider_ident: Ident) -> TokenStream {
        quote! {

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
            use module_macro_shared::item_modifier::ItemModifier;
        }.into();
        imports
    }
}
