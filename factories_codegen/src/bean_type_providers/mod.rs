use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Path, Token};
use crate::provider::ProviderProvider;

pub struct BeanTypeProvider;

impl ProviderProvider for BeanTypeProvider {
    fn create_delegating_token_provider_tokens(
        provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
        path: &Vec<Path>
    ) -> TokenStream {
        quote! {
            use module_macro_shared::metadata_parser::ParseMetadataItem;

            pub struct DelegatingParseMetadataItem {
            }

            #(
                impl ParseMetadataItem<#path> for DelegatingParseMetadataItem {

                    fn parse_values(parse_container: &mut Option<Box<dyn MetadataItem>>) -> Option<&mut #path> {
                        #provider_type::parse_values(parse_container)
                    }
                }
            )*
        }
    }

    fn create_token_provider_tokens(provider_path: syn::Path, provider_ident: Ident) -> TokenStream {
        quote! {
            pub struct #provider_ident;

            impl ParseMetadataItem<#provider_path> for #provider_ident {
                fn parse_values(parse_container: &mut Option<Box<dyn MetadataItem>>) -> Option<&mut #provider_path> {
                    #provider_path::parse_values(parse_container)
                }
            }
        }
    }

    fn get_imports() -> TokenStream {
        quote!{
            use aspect_knockoff_provider;
            use module_macro_shared::parse_container::MetadataItem;
        }
    }
}