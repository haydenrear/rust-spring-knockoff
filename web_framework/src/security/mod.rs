#![feature(pattern)]

use std::collections::LinkedList;

pub mod test;
pub mod security {

    extern crate core;

    use crate::convert::{Registration, Registry};
    use crate::filter::filter::{Filter, FilterChain};
    use crate::request::request::{WebRequest, WebResponse};
    use crate::session::session::HttpSession;
    use alloc::string::String;
    use core::borrow::Borrow;
    use core::fmt::{Error, Formatter};
    use serde::{Deserialize, Serialize, Serializer};
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
            request: WebRequest,
        ) -> Result<Box<Authentication>, AuthenticationConversionError>;
    }

    pub struct UsernamePasswordAuthenticationFilter {}

    impl Default for UsernamePasswordAuthenticationFilter {
        fn default() -> Self {
            Self {}
        }
    }

    impl Filter for UsernamePasswordAuthenticationFilter {
        fn filter(&self, request: &WebRequest, response: &mut WebResponse, filter: FilterChain) {
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
            request: WebRequest,
        ) -> Result<Box<Authentication>, AuthenticationConversionError> {
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
        fn supports(&self, authentication_token: TypeId) -> bool;
        fn authenticate(&self, auth_token: Box<AuthenticationToken>) -> bool;
    }

    pub struct UsernamePasswordAuthenticationProvider {}

    impl AuthenticationProvider for UsernamePasswordAuthenticationProvider {
        fn supports(&self, authentication_token: TypeId) -> bool {
            // authentication_token == UsernamePasswordAuthenticationToken::get_type(String::from("UsernamePasswordAuthenticationToken"))
            todo!()
        }

        fn authenticate(&self, auth_token: Box<AuthenticationToken>) -> bool {
            todo!()
        }
    }

    impl DelegatingAuthenticationManager {
        fn authenticate(&self, auth_token: Box<AuthenticationToken>) -> bool {
            self.providers.iter().any(|provider| {
                if provider.supports(auth_token.type_id()) {
                    return provider.authenticate(auth_token.clone());
                }
                false
            })
        }
    }

    impl AuthenticationToken {
        fn auth(&self) -> Box<Authentication> {
            todo!()
        }
        fn name(&self) -> &'static str {
            todo!()
        }
    }

    impl Default for Authentication {
        fn default() -> Self {
            Self {
                authentication_type: AuthenticationType::default(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct AuthenticationToken {
        name: String,
        auth: Authentication,
    }

    impl Default for AuthenticationToken {
        fn default() -> Self {
            Self {
                name: String::default(),
                auth: Authentication::default()
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Authentication {
        authentication_type: AuthenticationType,
    }

    pub trait AuthenticationConverter: Converter<AuthenticationType, LinkedList<Authority>> + Send + Sync
    {
    }
    pub trait JwtAuthenticationConverter: AuthenticationConverter {}
    pub trait UsernamePasswordAuthenticationConverter: AuthenticationConverter {}
    pub trait OpenSamlAuthenticationConverter: AuthenticationConverter {}

    #[derive(Clone)]
    pub struct AuthenticationConverterRegistry {
        converters: LinkedList<&'static dyn AuthenticationConverter>,
    }

    //TODO: macro in app context builder for having user provided jwt authentication converter, or
    // other authentication converter to implement Registration<UserProvidedJwt> for JwtAuthenticationConverterRegistry
    // and also it will add it - the registry![userProvided] will go inside of the app context register
    impl Registry<dyn AuthenticationConverter> for AuthenticationConverterRegistry {
        fn read_only_registrations(&self) -> Box<LinkedList<&'static dyn AuthenticationConverter>> {
            Box::new(self.converters.clone())
        }
    }

    pub trait Converter<From, To> {
        fn convert(&self, from: &From) -> To;
        fn supports(&self, auth_type: AuthenticationType) -> bool;
    }

    impl Authentication {
        fn new(authentication_type: AuthenticationType) -> Self {
            return Self {
                authentication_type: authentication_type,
            };
        }

        fn get_authorities(&self) -> LinkedList<Authority> {
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
        fn set_principal(principal: String) {}
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Authority {
        authority: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum AuthenticationType {
        Jwt(JwtToken),
        SAML(OpenSamlAssertion),
        Password(UsernamePassword),
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct JwtToken {
        token: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct OpenSamlAssertion {
        assertion: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct UsernamePassword {
        username: String,
        password: String,
    }

    impl Default for AuthenticationType {
        fn default() -> Self {
            AuthenticationType::Password(UsernamePassword {
                username: String::default(),
                password: String::default(),
            })
        }
    }
}
