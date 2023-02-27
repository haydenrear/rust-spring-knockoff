#![feature(pattern)]
use std::collections::LinkedList;

pub mod test;
pub mod security_filter;
pub mod security_context_holder;
pub mod http_security;
pub mod authorization;
pub mod authentication;

pub mod security {

    extern crate core;

    use web_framework_shared::request::{EndpointMetadata, WebRequest};
    use crate::web_framework::convert::{AuthenticationConverterRegistry, Registration};
    use crate::web_framework::filter::filter::{Action, DelegatingFilterProxy};
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
    use data_framework::{Entity, Repo};
    use knockoff_security::knockoff_security::authentication_type::{AuthenticationConversionError, JwtToken, UsernamePassword};
    use knockoff_security::knockoff_security::user_request_account::{User, UserAccount};
    use module_macro_lib::AuthenticationType;
    use web_framework_shared::convert::Converter;
    use crate::web_framework::context::{ApplicationContext, RequestContext};
    use crate::web_framework::context_builder::AuthenticationConverterRegistryBuilder;
    use crate::web_framework::security::authentication::GrantedAuthority;

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
        pub p: PhantomData<&'a (dyn Any + Send + Sync)>,
        pub u: PhantomData<U>,
        pub repo: Box<R>,
    }

    impl <'a, R, U> UserDetailsService<U, String> for PersistenceUserDetailsService<'a, R, U>
        where
            U: UserAccount + Serialize + for<'de> Deserialize<'de> + Send + Sync,
            R: Repo<'a, U, String> {
        async fn load_by_username(&self, id: String) -> Option<U> {
            self.repo.find_by_id(id).await
        }
    }
}
