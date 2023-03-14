use core::fmt::Debug;
use knockoff_security::knockoff_security::user_request_account::UserAccount;
use authentication_gen::{AuthenticationType, AuthenticationTypeConverter};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use futures::executor;
use knockoff_security::knockoff_security::authentication_type::{AuthenticationAware, AuthenticationConversionError};
use web_framework_shared::authority::GrantedAuthority;
use web_framework_shared::convert::Converter;
use web_framework_shared::request::WebRequest;
use crate::web_framework::convert::{AuthenticationConverterRegistry, Registration};
use crate::web_framework::security::password::PasswordEncoder;
use crate::web_framework::security::user_details::UserDetailsService;

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


pub trait AuthenticationProvider: Send + Sync {
    fn supports(&self, authentication_token: &AuthenticationType) -> bool;
    fn authenticate(&self, auth_token: &mut AuthenticationToken) -> AuthenticationToken;
}

pub struct DaoAuthenticationProvider<U, UDS>
    where
        U: UserAccount + Serialize + for<'a> Deserialize<'a> + Send + Sync,
        UDS: UserDetailsService<U, String>
{
    pub user_details_service: UDS,
    pub password_encoder: Box<dyn PasswordEncoder>,
    pub phantom_user: PhantomData<U>
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
                executor::block_on(self.user_details_service.load_by_username(&username_password.username))
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


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AuthenticationToken {
    pub name: String,
    pub auth: AuthenticationType,
    pub authenticated: bool,
    pub authorities: Vec<GrantedAuthority>
}

/// Represents some details, like IP address, certificate serial number, etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuthenticationDetails {
    WebAuthenticationDetails(WebAuthenticationDetails),
    PreAuthenticatedAuthenticationDetails(PreAuthenticatedGrantedAuthoritiesWebAuthenticationDetails)
}

impl Default for AuthenticationDetails {
    fn default() -> Self {
        AuthenticationDetails::WebAuthenticationDetails(WebAuthenticationDetails::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WebAuthenticationDetails {
    pub remote_address: String,
    pub session_id: String
}

/// Could be x509 pre-authentication.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PreAuthenticatedGrantedAuthoritiesWebAuthenticationDetails {
    pub granted_authorities: Vec<GrantedAuthority>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Authentication {
    pub principal: String,
    pub credentials: AuthenticationToken,
    pub details: AuthenticationDetails,
    pub authenticated: bool,
    pub authorities: Vec<GrantedAuthority>
}

pub trait AuthenticationConverter: Converter<WebRequest, Result<Authentication, AuthenticationConversionError>> + Send + Sync {
}

impl Converter<(&AuthenticationToken, &WebRequest), Result<Authentication, AuthenticationConversionError>> for AuthenticationConverterRegistry {
    fn convert(&self, from: &(&AuthenticationToken, &WebRequest))
        -> Result<Authentication, AuthenticationConversionError>
    {
        let auth_token = from.0;
        self.convert(&(from.1, auth_token)).map(|auth_details| {
            Authentication {
                principal: auth_token.name.clone(),
                credentials: auth_token.clone(),
                details: auth_details,
                authenticated: auth_token.authenticated.clone(),
                authorities: auth_token.authorities.clone(),
            }
        }).or(Ok::<Authentication, AuthenticationConversionError>(
            Authentication{
                principal: auth_token.name.clone(),
                credentials: auth_token.clone(),
                details: AuthenticationDetails::default(),
                authenticated: auth_token.authenticated.clone(),
                authorities: auth_token.authorities.clone(),
            }
        ))
    }
}

pub trait AuthenticationDetailsConverter<'a>: Converter<(&'a WebRequest, &'a AuthenticationToken), Result<AuthenticationDetails, AuthenticationConversionError>> + Send + Sync {
}

impl Converter<(&WebRequest, &AuthenticationToken), Result<AuthenticationDetails, AuthenticationConversionError>> for AuthenticationConverterRegistry {
    fn convert(&self, from: &(&WebRequest, &AuthenticationToken)) -> Result<AuthenticationDetails, AuthenticationConversionError> {
        Err(AuthenticationConversionError::default())
    }
}

impl <'a> AuthenticationDetailsConverter<'a> for AuthenticationConverterRegistry {
}
