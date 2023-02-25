use std::collections::LinkedList;
use serde::{Deserialize, Serialize};
use web_framework_shared::request::WebRequest;
use web_framework_shared::convert::Converter;

pub struct AuthenticationConversionError {
    pub message: String,
}

impl AuthenticationConversionError {
    pub fn new(message: String) -> Self {
        Self { message: message }
    }
}

pub trait AuthenticationAware {
    fn get_authorities(&self) -> LinkedList<Authority>;
    fn get_credentials(&self) -> Option<String>;
    fn get_principal(&self) -> Option<String>;
    fn set_credentials(&mut self, credential: String);
    fn set_principal(&mut self, principal: String);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authority {
    authority: String,
}

// impl AuthType for AuthenticationType {
//     fn parse_credentials(&self, request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
//         Err(AuthenticationConversionError::new(String::from("Authentication type was empty.")))
//     }
// }
//
// //TODO: each authentication provider is of generic type AuthType, allowing for generalization
// // then when user provides authentication provider overriding getAuthType with own, macro adds
// // the authentication provider to the map of auth providers in the authentication filter
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub enum AuthenticationType
// {
//     Jwt(JwtToken),
//     SAML(OpenSamlAssertion),
//     Password(UsernamePassword),
//     Unauthenticated
// }
//
// impl AuthenticationAware for AuthenticationType {
//     fn get_authorities(&self) -> LinkedList<Authority> {
//         todo!()
//     }
//
//     fn get_credentials(&self) -> Option<String> {
//         todo!()
//     }
//
//     fn get_principal(&self) -> Option<String> {
//         todo!()
//     }
//
//     fn set_credentials(&mut self, credential: String) {
//         todo!()
//     }
//
//     fn set_principal(&mut self, principal: String) {
//         todo!()
//     }
// }

pub struct AuthHelper;

impl AuthHelper {
    fn get_authorization_header_split(request: &WebRequest, auth_header_name: Option<String>) -> Option<String> {
        if request.headers.contains_key("Authorization") {
            let auth_string = request.headers["Authorization"].clone();
            let auth_header = auth_string.split("Authorization").collect::<Vec<_>>();
            return auth_header_name.map(|auth_header_name| {
                let bearer = auth_header[1].split(&auth_header_name).collect::<Vec<&str>>();
                if bearer.len() > 1 {
                    return Some(String::from(bearer[1].trim_end_matches(" ").trim_start_matches(" ")))
                }
                None
            }).or(Some(Some(auth_string))).flatten();
        }
        None
    }
}

pub trait AuthType: AuthenticationAware + Send + Sync + Default {

    fn parse_credentials(&self, request: &WebRequest) -> Result<Self, AuthenticationConversionError>;

}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtToken {
    pub token: String,
}

impl AuthenticationAware for JwtToken {
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

impl Default for JwtToken {
    fn default() -> Self {
        todo!()
    }
}

impl AuthType for JwtToken {
    fn parse_credentials(&self, request: &WebRequest) -> Result<JwtToken, AuthenticationConversionError> {
        JwtToken::parse_credentials_jwt(request)
    }
}

impl JwtToken {
    pub fn parse_credentials_jwt(request: &WebRequest) -> Result<JwtToken, AuthenticationConversionError> {
        AuthHelper::get_authorization_header_split(request, Some(String::from("Bearer")))
            .map(|bearer_token| {
                return Ok(JwtToken{ token: bearer_token})
            })
            .or(Some(Err(AuthenticationConversionError::new(String::from("Bearer token did not exist.")))))
            .unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unauthenticated {
}

impl AuthenticationAware for Unauthenticated {
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

impl Default for Unauthenticated {
    fn default() -> Self {
        todo!()
    }
}

impl AuthType for Unauthenticated {
    fn parse_credentials(&self, request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
        Unauthenticated::parse_credentials_unauthenticated(request)
    }
}

impl Unauthenticated {
    fn parse_credentials_unauthenticated(request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
        Ok(Self{})
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenSamlAssertion {
    assertion: String,
}

impl AuthenticationAware for OpenSamlAssertion {
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

impl Default for OpenSamlAssertion {
    fn default() -> Self {
        todo!()
    }
}

impl AuthType for OpenSamlAssertion {
    fn parse_credentials(&self, request: &WebRequest) -> Result<OpenSamlAssertion, AuthenticationConversionError> {
        OpenSamlAssertion::parse_credentials_opensaml(request)
    }
}

impl OpenSamlAssertion {
    fn parse_credentials_opensaml(request: &WebRequest) -> Result<OpenSamlAssertion, AuthenticationConversionError> {
        todo!()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsernamePassword {
    pub username: String,
    pub password: String,
}

impl AuthenticationAware for UsernamePassword {
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

impl Default for UsernamePassword {
    fn default() -> Self {
        todo!()
    }
}

impl AuthType for UsernamePassword {
    fn parse_credentials(&self, request: &WebRequest) -> Result<UsernamePassword, AuthenticationConversionError> {
        Self::parse_credentials_inner(request)
    }
}

impl UsernamePassword {

    pub fn parse_credentials_inner(request: &WebRequest) -> Result<UsernamePassword, AuthenticationConversionError> {
        AuthHelper::get_authorization_header_split(request, Some(String::from("Basic")))
            .map(|basic_auth_header| {
                Self::parse_username_password(basic_auth_header)
            })
            .or(Some(Err(AuthenticationConversionError::new(String::from(String::from("Failed to find auth header"))))))
            .unwrap()
    }

    fn parse_username_password(auth_string: String) -> Result<UsernamePassword, AuthenticationConversionError> {
        let mut auth_header = auth_string.as_str();

        let found = auth_header.split(":").collect::<Vec<&str>>();

        let username64 = found[0];
        let password64 = found[1];

        let username_result = base64::decode(username64);
        let password_result = base64::decode(password64);

        if username_result.is_err() {
            return Err(AuthenticationConversionError::new(String::from("Username could not be decoded")));
        }
        if password_result.is_err() {
            return Err(AuthenticationConversionError::new(String::from("Password could not be decoded")));
        }
        let username = String::from_utf8(username_result.unwrap())
            .unwrap();
        let password = String::from_utf8(password_result.unwrap())
            .unwrap();
        return Ok(UsernamePassword { username, password });
    }
}

