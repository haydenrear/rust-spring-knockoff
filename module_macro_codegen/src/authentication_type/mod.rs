use std::{env, fs};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Error;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use quote::__private::ext::RepToTokensExt;
use syn::{Attribute, Item, ItemFn, ItemImpl, ItemStruct, Type};
use syn::__private::str;
use syn::token::Token;
use knockoff_logging::use_logging;
use crate::parser::{CodegenItem, CodegenItemType, LibParser};

use_logging!();

use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

#[derive(Clone, Default)]
pub struct AuthenticationTypeCodegen {
    default: Option<TokenStream>,
    item: Vec<Item>,
}

impl AuthenticationTypeCodegen {

    pub(crate) fn new_dyn_codegen(item: &Vec<Item>) -> Option<CodegenItemType> {
        Self::new(item)
            .map(|i| CodegenItemType::AuthenticationType(i))
    }

    pub(crate) fn new(item: &Vec<Item>) -> Option<Self> {
        if AuthenticationTypeCodegen::supports_item(item) {
            return Some(Self { default: None, item: item.clone().iter().map(|v| v.clone()).collect() });
        }
        None
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

    fn add_item_struct(mut to_add_map: &mut HashMap<String, NextAuthType>, struct_found: &ItemStruct) {
        struct_found.attrs.iter().for_each(|attr| {
            log_message!("{} is the path.", attr.path.to_token_stream().to_string().as_str());
            log_message!("{} is the other.", attr.tokens.to_token_stream().to_string().as_str());
        });
        let id = struct_found.ident.to_token_stream().to_string().clone();
        let struct_opt_to_add = Some(struct_found.clone());
        if to_add_map.contains_key(&id) {
            to_add_map.get_mut(&id).map(|f| {
                f.auth_type_to_add = struct_opt_to_add
            });
        } else {
            let next = NextAuthType {
                auth_type_to_add: struct_opt_to_add,
                auth_type_impl: None,
                auth_aware_impl: None,
            };
            to_add_map.insert(id, next);
        }
    }

    fn insert_item_impl(mut to_add_map: &mut HashMap<String, NextAuthType>, impl_found: &&ItemImpl) {
        let id = impl_found.self_ty.clone().to_token_stream().to_string();
        if to_add_map.contains_key(&id) {
            Self::add_item_impl(&mut to_add_map, &id, &impl_found)
        } else {
            to_add_map.insert(id.clone(), NextAuthType {
                auth_type_to_add: None,
                auth_type_impl: None,
                auth_aware_impl: None,
            });
            Self::add_item_impl(&mut to_add_map, &id, &impl_found)
        }
    }

    fn add_item_to_map(mut to_add_map: &mut HashMap<String, NextAuthType>, item_to_create: &Item) {
        match item_to_create {
            Item::Struct(struct_found) => {
                Self::add_item_struct(&mut to_add_map, struct_found);
            }
            Item::Impl(impl_found) => {
                Self::insert_item_impl(&mut to_add_map, &impl_found);
            }
            _ => {}
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
                use web_framework_shared::authority::GrantedAuthority;
                use knockoff_security::knockoff_security::authentication_type::{
                    UsernamePassword, AuthenticationConversionError, JwtToken, AuthType,
                    OpenSamlAssertion , AuthenticationAware, Anonymous
                };
                use spring_knockoff_boot_macro::{auth_type_aware, auth_type_impl, auth_type_struct};
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

                impl AuthenticationType {
                    fn auth_type_action<T>(&self, action: &dyn Fn(&dyn AuthenticationAware) -> T, default: T) -> T {
                        match self {
                            #(AuthenticationType::#types(auth_type) => {
                                action(auth_type as &dyn AuthenticationAware)
                            })*
                            AuthenticationType::Jwt(jwt) => {
                                action(jwt as &dyn AuthenticationAware)
                            }
                            AuthenticationType::SAML(saml) => {
                                action(saml as &dyn AuthenticationAware)
                            }
                            AuthenticationType::Password(password) => {
                                action(password as &dyn AuthenticationAware)
                            }
                            AuthenticationType::Unauthenticated(anon) => {
                                action(anon as &dyn AuthenticationAware)
                            }
                            _ => {
                                default
                            }
                        }
                    }

                    fn auth_type_action_no_return(&mut self, action: Box<dyn FnOnce(&mut dyn AuthenticationAware)>) {
                        match self {
                            #(AuthenticationType::#types(auth_type) => {
                                action(auth_type as &mut dyn AuthenticationAware);
                            })*
                            AuthenticationType::Jwt(jwt) => {
                                action(jwt as &mut dyn AuthenticationAware);
                            }
                            AuthenticationType::SAML(saml) => {
                                action(saml as &mut dyn AuthenticationAware);
                            }
                            AuthenticationType::Password(password) => {
                                action(password as &mut dyn AuthenticationAware);
                            }
                            AuthenticationType::Unauthenticated(anon) => {
                                action(anon as &mut dyn AuthenticationAware);
                            }
                        };
                    }
                }

                ///TODO: macro..
                impl AuthenticationAware for AuthenticationType {
                    fn get_authorities(&self) -> Vec<GrantedAuthority> {
                        self.auth_type_action(&|auth: &dyn AuthenticationAware| auth.get_authorities().clone(), vec![])
                    }

                    fn get_credentials(&self) -> Option<String> {
                        self.auth_type_action(&|auth: &dyn AuthenticationAware| auth.get_credentials(), None)
                    }


                    fn get_principal(&self) -> Option<String> {
                        self.auth_type_action(&|auth: &dyn AuthenticationAware| auth.get_principal(), None)
                    }

                    fn set_credentials(&mut self, credential: String) {
                        self.auth_type_action_no_return(Box::new(|auth: &mut dyn AuthenticationAware| auth.set_credentials(credential)));
                    }

                    fn set_principal(&mut self, principal: String) {
                        self.auth_type_action_no_return(Box::new(|auth: &mut dyn AuthenticationAware| auth.set_principal(principal)));
                    }
                }

         };
        t.into()
    }

    fn create_prepare_auth_type_ts(types_next: &Vec<NextAuthType>) -> (Vec<Ident>, Vec<Type>, Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
        let enum_names = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_type_to_add.clone().unwrap().ident.clone(),
        );
        let types = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_type_impl.clone().unwrap().self_ty.deref().clone(),
        );
        let types_tokens = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_type_to_add.clone().unwrap().to_token_stream().clone(),
        );
        let impl_tokens = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_type_impl.clone().unwrap().to_token_stream().clone(),
        );
        let auth_aware = Self::get_collect_ts_type(
            types_next,
            &|next| next.auth_aware_impl.clone().unwrap().to_token_stream().clone(),
        );
        (enum_names, types, types_tokens, impl_tokens, auth_aware)
    }

    fn get_collect_ts_type<T: ToTokens>(types_next: &Vec<NextAuthType>, ts_getter: &dyn Fn(&NextAuthType) -> T) -> Vec<T> {
        types_next.iter()
            .map(|next| ts_getter(next))
            .collect::<Vec<T>>()
    }

    fn get_converter(additional_auth_types: &Vec<NextAuthType>) -> TokenStream {
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
                            #(#additional_auth_types::AUTH_TYPE => {
                                #additional_auth_types::parse_credentials(from)
                                    .map(|auth| AuthenticationType::#additional_auth_types(auth))
                            })*
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

    fn get_codegen_items(types: &Vec<NextAuthType>) -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::get_imports());
        ts.append_all(Self::get_authentication_types(&types));
        ts.append_all(Self::get_converter(&types));
        ts.into()
    }
}

#[derive(Default, Clone)]
struct NextAuthType {
    auth_type_to_add: Option<ItemStruct>,
    auth_type_impl: Option<ItemImpl>,
    auth_aware_impl: Option<ItemImpl>,
}

impl CodegenItem for AuthenticationTypeCodegen {

    fn supports_item(impl_item: &Vec<Item>) -> bool where Self: Sized {
        impl_item.iter().any(|impl_item| {
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
        })
    }

    fn supports(&self, item: &Vec<Item>) -> bool {
        Self::supports_item(item)
    }

    fn get_codegen(&self) -> Option<String> {
        if self.item.len() == 0 {
            return None;
        }

        let auth_types: Vec<NextAuthType> = self.item.iter().clone().flat_map(|item| {

            match item {
                Item::Mod(item_mod) => {
                    let mut to_add_map: HashMap<String, NextAuthType> = HashMap::new();

                    item_mod.content.iter().flat_map(|cnt| cnt.1.iter())
                        .for_each(|item_to_create|
                            Self::add_item_to_map(&mut to_add_map, item_to_create)
                        );

                    let auth_types = to_add_map.values()
                        .map(|next| next.to_owned().clone())
                        .collect::<Vec<NextAuthType>>();
                    auth_types

                }
                _ => {
                    vec![]
                }
            }
        })
            .collect();


        let tokens = Self::get_codegen_items(&auth_types);

        let q = quote! {
                        #tokens
                    };
        Some(q.to_string())
    }

    fn default_codegen(&self) -> String {
        AuthenticationTypeCodegen::default_tokens().to_string()
    }

    fn get_unique_id(&self) -> String {
        String::from("AuthenticationItem")
    }
}
