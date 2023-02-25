use std::{env, fs};
use std::fmt::Error;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Item, ItemFn, ItemImpl};
use crate::parser::{CodegenItem, LibParser};

#[derive(Clone)]
pub struct AuthenticationType {
    default: Option<TokenStream>
}

impl AuthenticationType {
    pub(crate) fn new() -> Self {
        Self {
            default: None
        }
    }
}


impl AuthenticationType {
    fn default_tokens() -> TokenStream {
        let t = quote! {

                use std::collections::LinkedList;
                use serde::{Deserialize, Serialize};
                use web_framework_shared::request::WebRequest;
                use web_framework_shared::convert::Converter;
                use knockoff_security::knockoff_security::authentication_type::{
                    UsernamePassword, AuthenticationConversionError, JwtToken, AuthType,
                    OpenSamlAssertion , AuthenticationAware, Authority
                };

                impl  Default for AuthenticationType {
                    fn default() -> Self {
                        AuthenticationType::Unauthenticated
                    }
                }

                pub trait AuthenticationTypeConverter: Converter<WebRequest, Result<AuthenticationType, AuthenticationConversionError>> + Send + Sync {
                }

                #[derive(Clone)]
                pub struct AuthenticationTypeConverterImpl {
                }

                impl AuthenticationTypeConverterImpl {
                    pub fn new() -> Self {
                        Self {
                        }
                    }

                }

                impl Converter<WebRequest, Result<AuthenticationType, AuthenticationConversionError>> for AuthenticationTypeConverterImpl {

                    fn convert(&self, from: &WebRequest) -> Result<AuthenticationType, AuthenticationConversionError> {
                        let auth_header = from.headers["Authorization"].as_str();
                        let first_split: Vec<&str> = auth_header.split_whitespace().collect();
                        if first_split.len() < 2 {
                            return Ok(AuthenticationType::Unauthenticated);
                        }
                        match first_split[0] {
                            "Basic" => {
                                UsernamePassword::parse_credentials_inner(from)
                                    .map(|auth| AuthenticationType::Password (auth))
                            }
                            "Bearer" => {
                                JwtToken::parse_credentials_jwt(from)
                                    .map(|auth| AuthenticationType::Jwt(auth))
                            }
                            _ => Ok(AuthenticationType::Unauthenticated)
                        }
                    }

                }

                impl AuthenticationTypeConverter for AuthenticationTypeConverterImpl {
                }

                impl AuthType for AuthenticationType {
                    fn parse_credentials(&self, request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
                        Err(AuthenticationConversionError::new(String::from("Authentication type was empty.")))
                    }
                }

                //TODO: each authentication provider is of generic type AuthType, allowing for generalization
                // then when user provides authentication provider overriding getAuthType with own, macro adds
                // the authentication provider to the map of auth providers in the authentication filter
                #[derive(Clone, Debug, Serialize, Deserialize)]
                pub enum AuthenticationType
                {
                    Jwt(JwtToken),
                    SAML(OpenSamlAssertion),
                    Password(UsernamePassword),
                    Unauthenticated
                }

                impl AuthenticationAware for AuthenticationType {
                    fn get_authorities(&self) -> LinkedList<Authority> {
                        todo!()
                    }

                    fn get_credentials(&self) -> Option<String> {
                        todo!()
                    }

                    fn get_principal(&self) -> Option<String> {
                        todo!()
                    }

                    fn set_credentials(&mut self, credential: String) {
                        todo!()
                    }

                    fn set_principal(&mut self, principal: String) {
                        todo!()
                    }
                }
            }.into();
        t
    }
}

impl CodegenItem for AuthenticationType {
    fn supports(&self, impl_item: &Item) -> bool {
        match impl_item {
            Item::Enum(impl_item) => {
                impl_item.attrs.iter()
                    .any(|attr_found| attr_found.to_token_stream()
                        .to_string().as_str().contains("authentication_type")
                    )
            }
            _ => {
                false
            }
        }
    }

    fn get_codegen(&self, item_fn: &Item) -> Option<String> {
        match item_fn {
            Item::Enum(item_fn) => {
                let q = quote! {
                    #item_fn
                };
                Some(q.to_string())
            }
            _ => {
                None
            }
        }
    }

    fn default_codegen(&self) -> String {
        AuthenticationType::default_tokens().to_string()
    }

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        Box::new(self.clone())
    }

    fn get_unique_id(&self) -> String {
        String::from("AuthenticationItem")
    }
}
