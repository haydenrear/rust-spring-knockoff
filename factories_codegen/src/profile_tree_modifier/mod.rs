use crate::provider::ProviderProvider;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Path;

pub struct ProfileTreeModifierProvider;

/**
Generate the ProfileTree based on BeanDefinitions added in the ParseContainerModifier.
*/
impl ProviderProvider for ProfileTreeModifierProvider {
    fn create_delegating_token_provider_tokens(provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
                                               path: &Vec<Path>) -> TokenStream {
        quote! {

            pub struct DelegatingProfileTreeModifierProvider {
                #(#provider_idents: #provider_type,)*
            }

            impl ProfileTreeModifier for DelegatingProfileTreeModifierProvider {

                fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
                    #(
                        self.#provider_idents.modify_bean(dep_type, profile_tree);
                    )*
                }

                fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self {
                    #(
                        let #provider_idents = #provider_type::new(profile_tree_items);
                    )*
                    Self {
                        #(#provider_idents,)*
                    }
                }

            }

        }
    }

    fn create_token_provider_tokens<T: ToTokens>(use_statement: T, builder_path: syn::Path, provider_ident: Ident) -> TokenStream {
        quote! {

            #use_statement

            pub struct #provider_ident {
                d: #builder_path
            }

            impl #provider_ident {
                fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
                    self.d.modify_bean(dep_type, profile_tree);
                }

                fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self {
                    Self {
                        d: #builder_path::new(profile_tree_items)
                    }
                }
            }

        }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
            use module_macro_shared::*;
            use std::collections::HashMap;
        }.into();
        imports
    }
}

