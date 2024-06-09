use std::collections::HashMap;
use std::ops::Deref;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{Item, ItemImpl, ItemStruct, Type};
use crate::{AuthTypes, METADATA_ITEM_ID, METADATA_TYPE_ITEM_ID, NextAuthType};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use module_macro_shared::parse_container::MetadataItemId;
use module_macro_shared::profile_tree::ProfileTree;
use crate::logger_lazy;
import_logger!("authentication_gen_token_stream_provider.rs");

pub struct AuthenticationTypeTokenStreamGenerator {
    auth_types: Option<AuthTypes>
}

impl AuthenticationTypeTokenStreamGenerator {

    /// This generates the aspect.
    pub fn generate_token_stream(&self) -> TokenStream {
        self.auth_types.as_ref().map(|a| Self::get_codegen(a))
            .or(Some(Self::default_tokens()))
            .unwrap()
    }

    pub fn new(profile_tree: &mut ProfileTree) -> Self {
        Self {
            auth_types: profile_tree.provided_items.remove(&MetadataItemId::new(METADATA_ITEM_ID.into(),
                                                                                 METADATA_TYPE_ITEM_ID.into()))
                .into_iter().flat_map(|removed| removed.into_iter())
                .flat_map(|mut item| {
                    AuthTypes::parse_values(&mut Some(item))
                        .map(|f| f.clone())
                        .into_iter()
                })
                .next()
        }

    }

}

impl AuthenticationTypeTokenStreamGenerator {

    fn get_codegen(auth_types: &AuthTypes) -> TokenStream {
        let tokens = Self::get_codegen_items(auth_types);

        let q = quote! {
                #tokens
            };
        q
    }

    fn get_imports() -> TokenStream {
        let t = quote! {
                use std::collections::LinkedList;
                use web_framework_shared::convert::Converter;
                use web_framework_shared::authority::GrantedAuthority;
                use knockoff_security::knockoff_security::authentication_type::{
                    UsernamePassword, AuthenticationConversionError, JwtToken, AuthType,
                    OpenSamlAssertion , AuthenticationAware, Anonymous
                };
                use spring_knockoff_boot_macro::{auth_type_aware, auth_type_impl, auth_type_struct};
                use serde::{Serialize, Deserialize};
                use spring_knockoff_boot_macro::knockoff_ignore;
                use web_framework_shared::request::WebRequest;
        };
        t.into()
    }

    fn get_authentication_types(types_next: &Vec<NextAuthType>) -> TokenStream {
        let (enum_names, types, types_tokens, impl_tokens, auth_aware) =
            Self::create_prepare_auth_type_ts(types_next);

        let t = quote! {

            #(#types_tokens)*

            #(#impl_tokens)*

            #(#auth_aware)*

            impl Default for AuthenticationType {
                fn default() -> Self {
                    AuthenticationType::Unauthenticated(Anonymous::default())
                }
            }

            impl AuthType for AuthenticationType {
                const AUTH_TYPE: &'static str = "";
                fn parse_credentials(request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
                    Err(AuthenticationConversionError::new(String::from("Authentication type was empty.")))
                }
            }

            #[derive(Clone, Debug, Serialize, Deserialize)]
            pub enum AuthenticationType
            {
                Jwt(JwtToken),
                SAML(OpenSamlAssertion),
                Password(UsernamePassword),
                Unauthenticated(Anonymous),
                #(#enum_names(#types)),*
            }

            macro_rules! call {
                ($call:ident, $self_val:ident) => {
                    match $self_val {
                        #(AuthenticationType::#types(auth_type) => {
                            auth_type.$call()
                        })*
                        AuthenticationType::Jwt(auth_type) => {
                            auth_type.$call()
                        }
                        AuthenticationType::SAML(auth_type) => {
                            auth_type.$call()
                        }
                        AuthenticationType::Password(auth_type) => {
                            auth_type.$call()
                        }
                        AuthenticationType::Unauthenticated(auth_type) => {
                            auth_type.$call()
                        }
                    };
                };
                ($call:ident, $item:tt, $self_val:ident) => {
                    match $self_val {
                        #(AuthenticationType::#types(auth_type) => {
                            auth_type.$call($item)
                        })*
                        AuthenticationType::Jwt(auth_type) => {
                            auth_type.$call($item)
                        }
                        AuthenticationType::SAML(auth_type) => {
                            auth_type.$call($item)
                        }
                        AuthenticationType::Password(auth_type) => {
                            auth_type.$call($item)
                        }
                        AuthenticationType::Unauthenticated(auth_type) => {
                            auth_type.$call($item)
                        }
                    };
                };
            }


            macro_rules! impl_auth_type {
                () => {
                    impl AuthenticationAware for AuthenticationType {

                        fn get_authorities(&self) -> Vec<GrantedAuthority> {
                            call!(get_authorities, self)
                        }

                        fn get_credentials(&self) -> Option<String> {
                            call!(get_credentials, self)
                        }

                        fn get_principal(&self) -> Option<String> {
                            call!(get_principal, self)
                        }

                        fn set_credentials(&mut self, credential: String) {
                            call!(set_credentials, credential, self)
                        }

                        fn set_principal(&mut self, principal: String) {
                            call!(set_principal, principal, self)
                        }
                    }
                }
            }

            impl_auth_type!();

         };
        t.into()
    }

    fn create_prepare_auth_type_ts(types_next: &Vec<NextAuthType>) -> (Vec<Ident>, Vec<Type>, Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
        types_next.iter().for_each(|s| {
            if let Some(f) = &s.auth_type_to_add {
                info!("has auth type to add") ;
            } else {
                info!("does not have auth type to add") ;
            }
            if let Some(f) = &s.auth_type_impl {
                info!("has auth type to impl") ;
            } else {
                info!("does not have auth type impl") ;
            }
            if let Some(f) = &s.auth_aware_impl {
                info!("has auth type aware") ;
            } else {
                info!("does not have auth aware") ;
            }
        });
        let enum_names = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_type_to_add.clone().iter().flat_map(|c| vec![c.ident.clone()]).collect(),
        );
        let types = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_type_impl.clone().iter().flat_map(|c| vec![c.self_ty.deref().clone()]).collect(),
        );
        let types_tokens = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_type_to_add.clone().iter().flat_map(|t| vec![t.to_token_stream()]).collect(),
        );
        let impl_tokens = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_type_impl.clone().iter().flat_map(|t| vec![t.to_token_stream()]).collect(),
        );
        let auth_aware = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_aware_impl.clone().iter().flat_map(|t| vec![t.to_token_stream()]).collect(),
        );
        (enum_names, types, types_tokens, impl_tokens, auth_aware)
    }

    fn get_collect_ts_type<T: ToTokens>(types_next: &Vec<NextAuthType>, ts_getter: &dyn Fn(&NextAuthType) -> Vec<T>) -> Vec<T> {
        types_next.iter()
            .flat_map(|next| ts_getter(next))
            .collect::<Vec<T>>()
    }

    fn get_converter(additional_auth_types: &Vec<NextAuthType>) -> TokenStream {
        let additional_auth_types = additional_auth_types.iter()
            .flat_map(|auth| auth.auth_type_to_add.clone().iter().map(|i| i.ident.clone()).collect::<Vec<_>>())
            .collect::<Vec<Ident>>();

        let t = quote! {

                pub trait AuthenticationTypeConverter: Converter<WebRequest, Result<AuthenticationType, AuthenticationConversionError>> + Send + Sync {
                }

                #[derive(Clone)]
                pub struct AuthenticationTypeConverterImpl {
                }

                impl AuthenticationTypeConverterImpl {
                    pub fn new() -> Self {
                        Self {}
                    }
                }

                impl Converter<WebRequest, Result<AuthenticationType, AuthenticationConversionError>> for AuthenticationTypeConverterImpl {

                    fn convert(&self, from: &WebRequest) -> Result<AuthenticationType, AuthenticationConversionError> {
                        let auth_header = from.headers["Authorization"].as_str();
                        let first_split: Vec<&str> = auth_header.split_whitespace().collect();
                        if first_split.len() < 2 {
                            return Ok(AuthenticationType::Unauthenticated(Anonymous::default()));
                        }
                        match first_split[0] {
                            UsernamePassword::AUTH_TYPE => {
                                UsernamePassword::parse_credentials(from)
                                    .map(|auth| AuthenticationType::Password(auth))
                            }
                            JwtToken::AUTH_TYPE => {
                                JwtToken::parse_credentials(from)
                                    .map(|auth| AuthenticationType::Jwt(auth))
                            }
                            // #(#additional_auth_types::AUTH_TYPE => {
                            //     #additional_auth_types::parse_credentials(from)
                            //         .map(|auth| AuthenticationType::#additional_auth_types(auth))
                            // })*
                            _ => Ok(AuthenticationType::Unauthenticated(Anonymous::default()))
                        }
                    }

                }

                impl AuthenticationTypeConverter for AuthenticationTypeConverterImpl {
                }

            }.into();

        t
    }

    fn default_tokens() -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        ts.append_all(Self::get_authentication_types(&vec!()));
        ts.append_all(Self::get_converter(&vec!()));
        ts.into()
    }

    fn get_codegen_items(types: &AuthTypes) -> TokenStream {
        let types = &types.auth_types;
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        ts.append_all(Self::get_authentication_types(types));
        ts.append_all(Self::get_converter(types));
        ts.into()
    }
}


