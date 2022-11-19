#![feature(pattern)]
pub mod test;
pub mod security {

    extern crate core;

    use crate::filter::filter::{Filter, FilterChain};
    use crate::request::request::{HttpRequest, HttpResponse};
    use crate::session::session::HttpSession;
    use crate::type_mod::type_mod::{GetType, HTypeId};
    use alloc::string::String;
    use core::borrow::Borrow;
    use core::fmt::{Error, Formatter};
    use serde::{Deserialize, Serialize};
    use std::any::{Any, TypeId};
    use std::cell::RefCell;
    use std::collections::{HashMap, LinkedList};
    use std::ptr::null;
    use std::vec;

    pub struct DelegatingAuthenticationManager {
        providers: LinkedList<Box<dyn AuthenticationProvider>>,
    }

    pub trait AuthenticationFilter: Filter {
        fn try_convert_to_authentication(
            &self,
            request: HttpRequest,
        ) -> Result<Box<AuthenticationImpl>, AuthenticationConversionError>;
    }

    pub struct UsernamePasswordAuthenticationFilter {}

    impl Default for UsernamePasswordAuthenticationFilter {
        fn default() -> Self {
            Self {}
        }
    }

    impl Filter for UsernamePasswordAuthenticationFilter {
        fn filter(&self, request: &HttpRequest, response: &mut HttpResponse, filter: FilterChain) {
            todo!()
        }
    }

    pub struct AuthenticationConversionError {
        message: String,
    }

    impl AuthenticationConversionError {
        fn new(message: String) -> Self {
            Self { message: message }
        }
    }

    impl AuthenticationFilter for UsernamePasswordAuthenticationFilter {
        fn try_convert_to_authentication(
            &self,
            request: HttpRequest,
        ) -> Result<Box<AuthenticationImpl>, AuthenticationConversionError> {
            todo!()
            // if request.headers.contains_key("Authorization") {
            //
            //     let auth_string = request.headers["Authorization"].clone();
            //
            //     let mut auth_header = auth_string.as_str();
            //
            //     let found = auth_header.split(":").collect::<Vec<&str>>();
            //
            //     let username64 = found[0];
            //     let password64 = found[1];
            //
            //     let username = base64::decode(username64);
            //     let password = base64::decode(password64);
            //
            //     if username.is_err() {
            //         return Err(AuthenticationConversionError::new(String::from("Username could not be decoded")));
            //     }
            //     if password.is_err(){
            //         return Err(AuthenticationConversionError::new(String::from("Password could not be decoded")));
            //     }
            // return Ok(Box::new(AuthenticationImpl::new(String::from_utf8(username.unwrap()).unwrap(), String::from_utf8(password.unwrap()).unwrap())));
            // } else {
            //     return Err(AuthenticationConversionError::new(String::from(String::from("Failed to find auth header"))));
            // }
        }
    }

    pub trait AuthorizationManager {}

    pub trait AuthorizationDecision {}

    pub struct AuthorizationRegistry {}

    pub trait GrantedAuthority {}

    pub struct SimpleGrantedAuthority {
        authority: String,
    }

    pub trait AuthenticationProvider {
        fn supports(&self, authentication_token: HTypeId) -> bool;
        fn authenticate(&self, auth_token: Box<AuthenticationTokenImpl>) -> bool;
    }

    pub struct UsernamePasswordAuthenticationProvider {}

    impl<T: AuthenticationToken> GetType for T
    where
        T: ?Sized,
    {
        fn get_type(name: String) -> HTypeId {
            HTypeId::new(name)
        }
        fn get_type_self(&self) -> HTypeId {
            HTypeId::new(String::from(self.name()))
        }
    }

    impl AuthenticationProvider for UsernamePasswordAuthenticationProvider {
        fn supports(&self, authentication_token: HTypeId) -> bool {
            // authentication_token == UsernamePasswordAuthenticationToken::get_type(String::from("UsernamePasswordAuthenticationToken"))
            todo!()
        }

        fn authenticate(&self, auth_token: Box<AuthenticationTokenImpl>) -> bool {
            todo!()
        }
    }

    impl DelegatingAuthenticationManager {
        fn authenticate(&self, auth_token: Box<AuthenticationTokenImpl>) -> bool {
            self.providers.iter().any(|provider| {
                if provider.supports(auth_token.get_type_self()) {
                    return provider.authenticate(auth_token.clone());
                }
                false
            })
        }
    }

    pub trait AuthenticationToken {
        fn name(&self) -> &'static str;
        fn auth(&self) -> Box<AuthenticationImpl>;
        fn default() -> Self
        where
            Self: Sized;
    }

    impl AuthenticationToken for AuthenticationTokenImpl {
        fn auth(&self) -> Box<AuthenticationImpl> {
            todo!()
        }
        fn default() -> Self
        where
            Self: Sized,
        {
            Self {
                name: String::from("default"),
                auth: AuthenticationImpl::default(),
            }
        }
        fn name(&self) -> &'static str {
            todo!()
        }
    }

    impl Default for AuthenticationImpl {
        fn default() -> Self {
            Self {
                authentication_type: String::from("default"),
            }
        }
    }

    impl Authentication for AuthenticationImpl {
        fn get_authorities(&self) -> LinkedList<String> {
            todo!()
        }
        fn get_credentials(&self) -> Option<String> {
            todo!()
        }
        fn get_principal(&self) -> Option<String> {
            todo!()
        }
        fn set_credentials(credential: String) {
            todo!()
        }
        fn set_principal(principal: String) {
            todo!()
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AuthenticationTokenImpl {
        name: String,
        auth: AuthenticationImpl,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AuthenticationImpl {
        authentication_type: String,
    }

    impl AuthenticationImpl {
        fn new(authentication_type: String) -> Self {
            return Self {
                authentication_type: authentication_type,
            };
        }
    }

    pub trait Authentication {
        fn get_principal(&self) -> Option<String>;
        fn get_credentials(&self) -> Option<String>;
        fn set_credentials(credential: String);
        fn set_principal(principal: String);
        fn get_authorities(&self) -> LinkedList<String>;
    }
}
