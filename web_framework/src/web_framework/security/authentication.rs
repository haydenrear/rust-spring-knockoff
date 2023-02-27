use knockoff_security::knockoff_security::user_request_account::UserAccount;
use module_macro_lib::AuthenticationType;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;
use futures::executor;
use web_framework_shared::convert::Converter;
use crate::web_framework::security::security::{PasswordEncoder, UserDetailsService};

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GrantedAuthority {
    pub authority: String,
}

impl GrantedAuthority {
    pub fn get_authority<'a>(&'a self) -> &'a str {
        &self.authority
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


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AuthenticationToken {
    pub name: String,
    pub auth: AuthenticationType,
    pub authenticated: bool,
    pub authorities: Vec<GrantedAuthority>
}

pub trait AuthenticationConverter: Converter<AuthenticationType, AuthenticationToken> + Send + Sync
{
    fn supports(&self, auth_type: &AuthenticationType) -> bool;
}
