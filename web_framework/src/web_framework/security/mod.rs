#![feature(pattern)]

use std::collections::LinkedList;

pub mod test;
pub mod security_filter;
pub mod security_context_holder;

pub mod security {

    extern crate core;

    use web_framework_shared::request::{EndpointMetadata, WebRequest};
    use crate::web_framework::convert::{AuthenticationConverterRegistry, Registration};
    use crate::web_framework::filter::filter::{Action, FilterChain};
    use crate::web_framework::request::request::WebResponse;
    use crate::web_framework::session::session::HttpSession;
    use alloc::string::String;
    use core::borrow::Borrow;
    use core::fmt::{Error, Formatter};
    use serde::{Deserialize, Serialize, Serializer};
    use std::any::{Any, TypeId};
    use std::cell::RefCell;
    use std::collections::{HashMap, LinkedList};
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use std::sync::{Arc, Mutex};
    use std::vec;
    use knockoff_security::knockoff_security::authentication_type::{AuthenticationConversionError, JwtToken, UsernamePassword};
    use knockoff_security::knockoff_security::user_request_account::UserAccount;
    use module_macro_lib::AuthenticationType;
    use web_framework_shared::convert::Converter;
    use crate::web_framework::context::{ApplicationContext, RequestContext};
    use crate::web_framework::context_builder::AuthenticationConverterRegistryBuilder;

    #[derive(Clone, Default)]
    pub struct DelegatingAuthenticationManager {
        pub(crate) providers: Arc<Vec<Box<dyn AuthenticationProvider>>>,
    }

    impl DelegatingAuthenticationManager {

        pub(crate) fn new() -> Self {
            Self {
                providers: Arc::new(vec![])
            }
        }

    }

    pub trait AuthorizationManager {}

    pub trait AuthorizationDecision {}

    pub struct AuthorizationRegistry {}

    pub trait GrantedAuthority {}

    pub struct SimpleGrantedAuthority {
        authority: String,
    }

    pub trait AuthenticationProvider : Send + Sync {
        fn supports(&self, authentication_token: &AuthenticationType) -> bool;
        fn authenticate(&self, auth_token: &mut AuthenticationToken) -> AuthenticationToken;
    }

    #[derive(Clone)]
    pub struct UsernamePasswordAuthenticationProvider {}

    impl AuthenticationProvider for UsernamePasswordAuthenticationProvider {
        fn supports(&self, authentication_token: &AuthenticationType) -> bool {
            // authentication_token == UsernamePasswordAuthenticationToken::get_type(String::from("UsernamePasswordAuthenticationToken"))
            todo!()
        }

        fn authenticate(&self, auth_token: &mut AuthenticationToken) -> AuthenticationToken {
            todo!()
        }

    }

    impl AuthenticationProvider for DelegatingAuthenticationManager {

        fn supports(&self, authentication_token: &AuthenticationType) -> bool {
            self.providers.iter().any(|auth| auth.supports(authentication_token))
        }

        fn authenticate(&self, auth_token: &mut AuthenticationToken) -> AuthenticationToken {
            for provider in self.providers.iter() {
                if provider.supports(&auth_token.auth) {
                    return provider.authenticate(auth_token).to_owned();
                }
            }
            auth_token.to_owned()
        }
    }

    impl AuthenticationToken {
        fn auth(&self) -> AuthenticationType {
            todo!()
        }
        fn name(&self) -> &'static str {
            todo!()
        }

    }

    impl Default for Authentication {
        fn default() -> Self {
            Self {
                authentication_type: AuthenticationType::Unauthenticated,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct AuthenticationToken {
        pub name: String,
        pub auth: AuthenticationType,
        pub authenticated: bool
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Authentication {
        pub authentication_type: AuthenticationType
    }

    pub trait AuthenticationConverter: Converter<AuthenticationType, AuthenticationToken> + Send + Sync
    {
        fn supports(&self, auth_type: &AuthenticationType) -> bool;
    }

    impl Authentication {
        fn new(authentication_type: AuthenticationType) -> Self {
            return Self {
                authentication_type: authentication_type,
            };
        }
    }
}
