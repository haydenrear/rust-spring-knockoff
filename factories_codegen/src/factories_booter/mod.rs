use crate::provider::ProviderProvider;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Path;

pub struct FactoryBootFrameworkTokenProvider;

/// Can refactor to pull out common with framework_token_provider
/// -- this is the same, except FactoryBootTokenProvider is generating tokens for the
///     booting process for codegen -
/// example:
///     codegen tokens for HandlerMapping is framework_token_provider
///     codegen tokens for booting up HandlerMapping and dispatcher at start is FactoryBootTokenProvider
impl ProviderProvider for FactoryBootFrameworkTokenProvider {
    fn create_delegating_token_provider_tokens(provider_type: Vec<Ident>, provider_idents: Vec<Ident>,
                                               path: &Vec<Path>) -> TokenStream {
        quote! {

            pub struct DelegatingFactoryBootTokenProvider {
                #(#provider_idents: #provider_type,)*
            }

            impl FactoryBootTokenProvider for DelegatingFactoryBootTokenProvider {
                fn new_boot(profile_tree: &mut ProfileTree) -> Self {
                    #(
                        let #provider_idents = #provider_type::new_boot(profile_tree);
                    )*
                    Self {
                        #(#provider_idents,)*
                    }
                }

                fn generate_boot_ts(&self) -> TokenStream {
                    let mut ts = TokenStream::default();
                    #(
                        ts.append_all(self.#provider_idents.generate_boot_ts());
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

                impl FactoryBootTokenProvider for #provider_ident {
                    fn new_boot(items: &mut ProfileTree) -> #provider_ident {
                        let profile_tree = items.clone();
                        let token_delegate = #builder_path::new_boot(items);
                        Self {
                            profile_tree,
                            token_delegate
                        }
                    }
                    fn generate_boot_ts(&self) -> TokenStream {
                        self.token_delegate.generate_boot_ts()
                    }
                }

        }
    }

    fn get_imports() -> TokenStream {
        let imports = quote! {
        }.into();
        imports
    }
}

