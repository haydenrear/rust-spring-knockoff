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
    use futures::{executor, FutureExt};
    use data_framework::Repo;
    use knockoff_security::knockoff_security::authentication_type::{AuthenticationConversionError, JwtToken, UsernamePassword};
    use knockoff_security::knockoff_security::user_request_account::{User, UserAccount};
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

    pub trait PasswordEncoder : Send + Sync {
        fn encode_password(&self, unencoded: &str) -> String;
    }

    #[derive(Clone)]
    pub struct NoOpPasswordEncoder;

    impl PasswordEncoder for NoOpPasswordEncoder {
        fn encode_password(&self, unencoded: &str) -> String {
            unencoded.to_string()
        }
    }


    pub trait UserDetailsService<U, ID>: Send + Sync
        where
            U: UserAccount + Serialize + for<'a> Deserialize<'a> + Send + Sync
    {
        async fn load_by_username(&self, id: ID) -> Option<U>;
    }

    pub struct PersistenceUserDetailsService<'a, R, U>
        where
            U: UserAccount + Serialize + for<'de> Deserialize<'de> + Send + Sync,
            R: Repo<'a, U, String> {
        p: &'a PhantomData<dyn Any + Send + Sync>,
        u: PhantomData<U>,
        repo: Box<R>,
    }

    impl <'a, R, U> UserDetailsService<U, String> for PersistenceUserDetailsService<'a, R, U>
        where
            U: UserAccount + Serialize + for<'de> Deserialize<'de> + Send + Sync,
            R: Repo<'a, U, String> {
        async fn load_by_username(&self, id: String) -> Option<U> {
            self.repo.find_by_id(id).await
        }
    }

    pub struct DaoAuthenticationProvider<U, UDS>
        where
            U: UserAccount + Serialize + for<'a> Deserialize<'a> + Send + Sync,
            UDS: UserDetailsService<U, String>
    {
        user_details_service: UDS,
        password_encoder: Box<dyn PasswordEncoder>,
        phantom_user: PhantomData<U>
    }

    impl <U, UDS> AuthenticationProvider for DaoAuthenticationProvider<U, UDS>
        where
            U: UserAccount + Serialize + for<'a> Deserialize<'a> + Send + Sync,
            UDS: UserDetailsService<U, String>
    {

        fn supports(&self, authentication_type: &AuthenticationType) -> bool {
            match authentication_type {
                AuthenticationType::Password(_) => {
                    true
                }
                _ => {
                    false
                }
            }
        }

        fn authenticate(&self, auth_token: &mut AuthenticationToken) -> AuthenticationToken {
            match auth_token.to_owned().auth {
                AuthenticationType::Password(username_password) => {
                    executor::block_on(self.user_details_service.load_by_username(username_password.username.clone()))
                        .map(|user_found| {
                            if self.password_encoder.encode_password(&username_password.username) == user_found.get_password() {
                                auth_token.authenticated = true;
                            }
                            auth_token
                        })
                        .or(Some(&mut AuthenticationToken::default()))
                        .unwrap().to_owned()
                }
                _ => {
                    auth_token.to_owned()
                }
            }
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
                authentication_type,
            };
        }
    }
}
