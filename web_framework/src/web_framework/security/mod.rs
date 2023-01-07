#![feature(pattern)]

use std::collections::LinkedList;

pub mod test;
pub mod security_filter;
pub mod security {

    extern crate core;

    use crate::web_framework::convert::{Registration, Registry};
    use crate::web_framework::filter::filter::{Action, FilterChain};
    use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
    use crate::web_framework::session::session::HttpSession;
    use alloc::string::String;
    use core::borrow::Borrow;
    use core::fmt::{Error, Formatter};
    use serde::{Deserialize, Serialize, Serializer};
    use std::any::{Any, TypeId};
    use std::cell::RefCell;
    use std::collections::{HashMap, LinkedList};
    use std::ptr::null;
    use std::vec;
    use security_model::UserAccount;
    use crate::web_framework::context::{ApplicationContext, RequestContext};

    pub struct DelegatingAuthenticationManager {
        pub(crate) providers: LinkedList<Box<dyn AuthenticationProvider>>,
    }

    impl Clone for DelegatingAuthenticationManager {
        fn clone(&self) -> Self {
            Self {
                providers: self.providers.iter()
                    .map(|a| a.clone_auth_provider())
                    .collect()
            }
        }
    }

    impl DelegatingAuthenticationManager {
        pub(crate) fn new() -> Self {
            Self {
                providers: LinkedList::new()
            }
        }
    }

    //TODO: replace filter with action
    pub trait AuthenticationFilter{
        fn try_convert_to_authentication(
            &self,
            request: &WebRequest,
        ) -> Option<Authentication>;
    }

    pub struct UsernamePasswordAuthenticationFilter {}

    impl Default for UsernamePasswordAuthenticationFilter {
        fn default() -> Self {
            Self {}
        }
    }

    pub trait DelegatingAuthenticationFilter {
        fn do_authentication();
    }

    impl <Request, Response> Action<Request, Response> for UsernamePasswordAuthenticationFilter
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
    fn do_action(
            &self,
            metadata: EndpointMetadata,
            request: &Option<Request>,
            web_request: &WebRequest,
            response: &mut WebResponse,
            context: &RequestContext,
            application_context: &ApplicationContext<Request, Response>
        ) -> Option<Response> {
            todo!()
        }

        fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
            todo!()
        }

        fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
            todo!()
        }

        fn clone(&self) -> Box<dyn Action<Request, Response>> {
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
            request: &WebRequest,
        ) -> Option<Authentication> {
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

    pub trait AuthenticationProvider : Send + Sync {
        fn supports(&self, authentication_token: TypeId) -> bool;
        fn authenticate(&self, auth_token: Box<AuthenticationToken>) -> bool;
        fn clone_auth_provider(&self) -> Box<dyn AuthenticationProvider>;
    }

    #[derive(Clone)]
    pub struct UsernamePasswordAuthenticationProvider {}

    impl AuthenticationProvider for UsernamePasswordAuthenticationProvider {
        fn supports(&self, authentication_token: TypeId) -> bool {
            // authentication_token == UsernamePasswordAuthenticationToken::get_type(String::from("UsernamePasswordAuthenticationToken"))
            todo!()
        }

        fn authenticate(&self, auth_token: Box<AuthenticationToken>) -> bool {
            todo!()
        }

        fn clone_auth_provider(&self) -> Box<dyn AuthenticationProvider> {
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
        auth: Authentication
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
        authentication_type: AuthenticationType
    }

    pub trait AuthenticationConverter: Converter<AuthenticationType, AuthenticationToken> + Send + Sync {
        fn supports(&self, auth_type: AuthenticationType) -> bool;
    }

    pub trait JwtAuthenticationConverter: AuthenticationConverter {}
    pub trait UsernamePasswordAuthenticationConverter: AuthenticationConverter {}
    pub trait OpenSamlAuthenticationConverter: AuthenticationConverter {}

    #[derive(Clone)]
    pub struct AuthenticationConverterRegistry {
        converters: LinkedList<&'static dyn AuthenticationConverter>,
        authentication_type_converter: &'static dyn AuthenticationTypeConverter
    }

    impl Converter<WebRequest, AuthenticationType> for AuthenticationConverterRegistry{
        fn convert(&self, from: &WebRequest) -> AuthenticationType {
            self.authentication_type_converter.convert(from)
        }
    }

    impl Converter<WebRequest, AuthenticationToken> for AuthenticationConverterRegistry{
        fn convert(&self, from: &WebRequest) -> AuthenticationToken {
            let authentication_type = self.authentication_type_converter.convert(from);
            AuthenticationToken {
                name: authentication_type.get_principal().unwrap_or("".to_string()),
                auth: Authentication {
                    authentication_type,
                },
            }
        }
    }

    impl <'a> Registration<'a, dyn AuthenticationConverter> for AuthenticationConverterRegistry
    where
        'a: 'static
    {
        fn register(&mut self, converter: &'a dyn AuthenticationConverter) {
            self.converters.push_back(converter)
        }
    }

    impl AuthenticationConverterRegistry {
        pub fn new() -> Self {
            Self {
                converters: LinkedList::new(),
                authentication_type_converter: &AuthenticationTypeConverterImpl {}
            }
        }
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
    }

    pub trait AuthenticationAware {
        fn get_authorities(&self) -> LinkedList<Authority>;
        fn get_credentials(&self) -> Option<String>;
        fn get_principal(&self) -> Option<String>;
        fn set_credentials(&mut self, credential: String);
        fn set_principal(&mut self, principal: String);
    }

    impl Authentication {
        fn new(authentication_type: AuthenticationType) -> Self {
            return Self {
                authentication_type: authentication_type,
            };
        }
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Authority {
        authority: String,
    }

    pub trait AuthenticationTypeConverter: Converter<WebRequest, AuthenticationType> + Send + Sync {
    }

    #[derive(Clone)]
    pub struct AuthenticationTypeConverterImpl;

    impl Converter<WebRequest, AuthenticationType> for AuthenticationTypeConverterImpl {
        fn convert(&self, from: &WebRequest) -> AuthenticationType {
            let auth_header = from.headers["Authorization"].as_str();
            let first_split: Vec<&str> = auth_header.split_whitespace().collect();
            if first_split.len() < 2 {
                return AuthenticationType::Unauthenticated;
            }
            match first_split[0] {
                "Basic" => {
                    let token = String::from(first_split[1]);
                    AuthenticationType::Jwt(JwtToken{ token })
                }
                "Bearer" => {
                    let username = "".to_string();
                    let password = "".to_string();
                    AuthenticationType::Password(UsernamePassword{username, password})
                }
                _ => AuthenticationType::Unauthenticated
            }
        }
    }

    impl AuthenticationTypeConverter for AuthenticationTypeConverterImpl {
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

    impl AuthType for AuthenticationType {

    }

    pub trait AuthType: AuthenticationAware + Send + Sync {

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
        pub(crate) username: String,
        pub(crate) password: String,
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
