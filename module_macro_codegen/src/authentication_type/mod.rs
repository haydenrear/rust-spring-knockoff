use std::{env, fs};
use std::collections::HashMap;
use std::fmt::Error;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use quote::__private::ext::RepToTokensExt;
use syn::{Item, ItemFn, ItemImpl, ItemStruct, Type};
use syn::__private::str;
use syn::token::Token;
use crate::parser::{CodegenItem, LibParser};

#[derive(Clone)]
pub struct AuthenticationTypeCodegen {
    default: Option<TokenStream>,
}

impl AuthenticationTypeCodegen {
    pub(crate) fn new() -> Self {
        Self {
            default: None
        }
    }

    fn add_item_impl(mut to_add_map: &mut HashMap<String, NextAuthType>, id: &String, impl_found: &ItemImpl) {
        if impl_found.trait_.clone()
            .map(|t| t.1.to_token_stream().to_string().as_str().contains("AuthType"))
            .filter(|f| *f)
            .or(Some(false))
            .unwrap() {
            to_add_map.get_mut(id).map(|f| f.auth_type_impl = Some(impl_found.clone()));
        } else if impl_found.trait_.clone()
            .map(|t| t.1.to_token_stream().to_string().as_str().contains("AuthenticationAware"))
            .filter(|f| *f)
            .or(Some(false))
            .unwrap() {
            to_add_map.get_mut(id).map(|f| f.auth_aware_impl = Some(impl_found.clone()));
        }
    }
}


impl AuthenticationTypeCodegen {
    fn get_imports() -> TokenStream {
        let t = quote! {
                use std::collections::LinkedList;
                use serde::{Deserialize, Serialize};
                use web_framework_shared::request::WebRequest;
                use web_framework_shared::convert::Converter;
                use knockoff_security::knockoff_security::authentication_type::{
                    UsernamePassword, AuthenticationConversionError, JwtToken, AuthType,
                    OpenSamlAssertion , AuthenticationAware, Authority
                };
        };
        t.into()
    }

    fn get_authentication_types(types_next: &Vec<&NextAuthType>) -> TokenStream {
        let enum_names = types_next.iter()
            .map(|next| next.auth_type_to_add.clone().unwrap().ident.clone())
            .collect::<Vec<Ident>>();
        let types = types_next.iter()
            .map(|next| next.auth_type_impl.clone().unwrap().self_ty.deref().clone())
            .collect::<Vec<Type>>();
        let types_tokens = types_next.iter()
            .map(|next| next.auth_type_to_add.clone().unwrap().to_token_stream().clone())
            .collect::<Vec<TokenStream>>();
        let impl_tokens = types_next.iter()
            .map(|next| next.auth_type_impl.clone().unwrap().to_token_stream().clone())
            .collect::<Vec<TokenStream>>();
        let auth_aware = types_next.iter()
            .map(|next| next.auth_aware_impl.clone().unwrap().to_token_stream().clone())
            .collect::<Vec<TokenStream>>();

        let t = quote! {

                #(#types_tokens)*

                #(#impl_tokens)*

                #(#auth_aware)*

                impl  Default for AuthenticationType {
                    fn default() -> Self {
                        AuthenticationType::Unauthenticated
                    }
                }

                impl AuthType for AuthenticationType {
                    const AUTH_TYPE: &'static str = "";
                    fn parse_credentials(request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
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
                    #(#enum_names(#types)),*,
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

         };
        t.into()
    }

    fn get_converter(additional_auth_types: &Vec<&NextAuthType>) -> TokenStream {
        let additional_auth_types = additional_auth_types.iter()
            .map(|auth| auth.auth_type_to_add.clone().unwrap().ident.clone())
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
                            return Ok(AuthenticationType::Unauthenticated);
                        }
                        match first_split[0] {
                            "Basic" => {
                                UsernamePassword::parse_credentials(from)
                                    .map(|auth| AuthenticationType::Password(auth))
                            }
                            "Bearer" => {
                                JwtToken::parse_credentials(from)
                                    .map(|auth| AuthenticationType::Jwt(auth))
                            }
                            #(#additional_auth_types::AUTH_TYPE => {
                                #additional_auth_types::parse_credentials(from)
                                    .map(|auth| AuthenticationType::#additional_auth_types(auth))
                            })*
                            _ => Ok(AuthenticationType::Unauthenticated)
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

    fn get_codegen_items(types: Vec<&NextAuthType>) -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        ts.append_all(Self::get_authentication_types(&types));
        ts.append_all(Self::get_converter(&types));
        ts.into()
    }
}

struct NextAuthType {
    auth_type_to_add: Option<ItemStruct>,
    auth_type_impl: Option<ItemImpl>,
    auth_aware_impl: Option<ItemImpl>
}

impl CodegenItem for AuthenticationTypeCodegen {
    fn supports(&self, impl_item: &Item) -> bool {
        match impl_item {
            Item::Mod(item_mod) => {
                item_mod.attrs.iter()
                    .any(|attr_found| attr_found.to_token_stream()
                        .to_string().as_str().contains("authentication_type")
                    )
            }
            _ => {
                false
            }
        }
    }

    fn get_codegen(&self, mod_item_to_match: &Item) -> Option<String> {
        match mod_item_to_match {
            Item::Mod(item_mod) => {
                let mut to_add_map: HashMap<String, NextAuthType> = HashMap::new();
                item_mod.content.iter().flat_map(|cnt| cnt.1.iter())
                    .for_each(|item_to_create| {
                        match item_to_create {
                            Item::Struct(struct_found) => {
                                let id = struct_found.ident.to_token_stream().to_string();
                                let mut struct_found = struct_found.clone();
                                if to_add_map.contains_key(&id) {
                                    to_add_map.get_mut(&id).map(|f| f.auth_type_to_add = Some(struct_found));
                                } else {
                                    let next = NextAuthType { auth_type_to_add: Some(struct_found), auth_type_impl: None, auth_aware_impl: None };
                                    to_add_map.insert(id, next);
                                }
                            }
                            Item::Impl(impl_found) => {
                                let id = impl_found.self_ty.clone().to_token_stream().to_string();
                                let impl_found = impl_found.clone();
                                if to_add_map.contains_key(&id) {
                                    Self::add_item_impl(&mut to_add_map, &id, &impl_found)
                                } else {
                                    to_add_map.insert(id.clone(), NextAuthType{
                                        auth_type_to_add: None,
                                        auth_type_impl: None,
                                        auth_aware_impl: None,
                                    });
                                    Self::add_item_impl(&mut to_add_map, &id, &impl_found)
                                }
                            }
                            _ => {}
                        }
                    });

                let auth_types = to_add_map.values()
                    .into_iter()
                    .collect::<Vec<&NextAuthType>>();

                let tokens = Self::get_codegen_items(auth_types);
                let q = quote! {
                    #tokens
                };
                Some(q.to_string())
            }
            _ => {
                None
            }
        }
    }

    fn default_codegen(&self) -> String {
        AuthenticationTypeCodegen::default_tokens().to_string()
    }

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        Box::new(self.clone())
    }

    fn get_unique_id(&self) -> String {
        String::from("AuthenticationItem")
    }
}
