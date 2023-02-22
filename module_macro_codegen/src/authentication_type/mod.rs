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

                extern crate core;

                // use crate::web_framework::convert::{AuthenticationConverterRegistry, Registration};
                // use crate::web_framework::filter::filter::{Action, FilterChain};
                // use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
                // use crate::web_framework::session::session::HttpSession;
                // use alloc::string::String;
                // use core::borrow::Borrow;
                // use core::fmt::{Error, Formatter};
                // use serde::{Deserialize, Serialize, Serializer};
                // use std::any::{Any, TypeId};
                // use std::cell::RefCell;
                // use std::collections::{HashMap, LinkedList};
                // use std::marker::PhantomData;
                // use std::ops::{Deref, DerefMut};
                // use std::ptr::null;
                // use std::sync::{Arc, Mutex};
                // use std::vec;
                // use security_model::UserAccount;
                // use crate::web_framework::context::{ApplicationContext, RequestContext};
                // use crate::web_framework::context_builder::AuthenticationConverterRegistryBuilder;

                // #[derive(Clone, Debug, Serialize, Deserialize)]
                // pub enum AuthenticationType
                // {
                //     Jwt(JwtToken),
                //     SAML(OpenSamlAssertion),
                //     Password(UsernamePassword),
                //     Unauthenticated
                // }
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
