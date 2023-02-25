use std::collections::LinkedList;
use std::fmt::{Display, Formatter};
use std::string::FromUtf8Error;
use base64::{DecodeError, Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use web_framework_shared::request::WebRequest;
use web_framework_shared::convert::Converter;

#[cfg(test)]
pub mod test;

#[derive(Debug, Default, Clone)]
pub struct AuthenticationConversionError {
    pub message: String,
}

impl Display for AuthenticationConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl AuthenticationConversionError {
    pub fn new(message: String) -> Self {
        Self { message: message }
    }
}

pub trait AuthenticationAware {
    fn get_authorities(&self) -> Vec<GrantedAuthority>;
    fn get_credentials(&self) -> Option<String>;
    fn get_principal(&self) -> Option<String>;
    fn set_credentials(&mut self, credential: String);
    fn set_principal(&mut self, principal: String);
}

pub struct AuthHelper;

impl AuthHelper {
    fn get_authorization_header_split(request: &WebRequest, auth_header_name: Option<String>) -> Option<String> {
        if request.headers.contains_key("Authorization") {
            let auth_string = request.headers["Authorization"].clone();
            return auth_header_name.map(|auth_header_name| {
                if auth_string.contains(&auth_header_name.to_lowercase()) {
                    return Self::parse_auth_token(
                        Self::get_auth_token(&auth_string, &auth_header_name.to_lowercase())
                    );
                } else {
                    return Self::parse_auth_token(
                        Self::get_auth_token(&auth_string, &auth_header_name)
                    );
                }
                None
            }).or(Some(Some(auth_string))).flatten();
        }
        None
    }

    fn parse_auth_token(auth_token: Vec<String>) -> Option<String> {
        if auth_token.len() > 1 {
            return Some(String::from(auth_token[1].trim_end_matches(" ").trim_start_matches(" ")))
        }
        None
    }

    fn get_auth_token(auth_string: &String, auth_header_name: &String) -> Vec<String> {
        let auth_token = auth_string
            .split(auth_header_name)
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        auth_token
    }
}

pub trait AuthType: AuthenticationAware + Send + Sync + Default {

    const AUTH_TYPE: &'static str;

    fn parse_credentials(request: &WebRequest) -> Result<Self, AuthenticationConversionError>;

    fn authorization_matcher(match_this: &str) -> bool {
        Self::match_this(match_this, Self::AUTH_TYPE)
    }

    fn match_this(match_this: &str, matching: &str) -> bool {
        match_this.to_lowercase().contains(matching)
        || matching.to_lowercase().contains(match_this)
    }

}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct JwtToken {
    pub token: String,
}

impl AuthenticationAware for JwtToken {
    fn get_authorities(&self) -> Vec<GrantedAuthority> {
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

impl AuthType for JwtToken {
    const AUTH_TYPE: &'static str = "bearer";

    fn parse_credentials(request: &WebRequest) -> Result<JwtToken, AuthenticationConversionError> {
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

/// TODO: extract this to codegen to define default authorities or anonymous auth filter.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Anonymous {
}

impl AuthenticationAware for Anonymous {
    fn get_authorities(&self) -> Vec<GrantedAuthority> {
        vec![]
    }

    fn get_credentials(&self) -> Option<String> {
        Some(String::default())
    }

    fn get_principal(&self) -> Option<String> {
        Some(String::default())
    }

    fn set_credentials(&mut self, credential: String) {
    }

    fn set_principal(&mut self, principal: String) {
    }
}

impl AuthType for Anonymous {
    const AUTH_TYPE: &'static str = "";

    fn parse_credentials(request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
        Anonymous::parse_credentials_unauthenticated(request)
    }

    fn authorization_matcher(match_this: &str) -> bool {
        true
    }
}

impl Anonymous {
    fn parse_credentials_unauthenticated(request: &WebRequest) -> Result<Self, AuthenticationConversionError> {
        Ok(Self{})
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenSamlAssertion {
    assertion: String,
}

impl AuthenticationAware for OpenSamlAssertion {
    fn get_authorities(&self) -> Vec<GrantedAuthority> {
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
    const AUTH_TYPE: &'static str = "research";

    fn parse_credentials(request: &WebRequest) -> Result<OpenSamlAssertion, AuthenticationConversionError> {
        OpenSamlAssertion::parse_credentials_opensaml(request)
    }

    fn authorization_matcher(match_this: &str) -> bool {
        // TODO:
        Self::match_this(match_this, "research")
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
    fn get_authorities(&self) -> Vec<GrantedAuthority> {
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
    const AUTH_TYPE: &'static str = "basic";

    fn parse_credentials(request: &WebRequest) -> Result<UsernamePassword, AuthenticationConversionError> {
        Self::parse_credentials_inner(request)
    }

    fn authorization_matcher(match_this: &str) -> bool {
        Self::match_this(match_this, Self::AUTH_TYPE)
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


        let username_result = general_purpose::STANDARD_NO_PAD.decode(username64.as_bytes());
        let password_result = general_purpose::STANDARD_NO_PAD.decode(password64.as_bytes());

        if username_result.is_err() {
            return Err(Self::err_decode(username_result.unwrap_err().to_string().as_str()).unwrap_err());
        }

        if password_result.is_err() {
            return Err(Self::err_decode(password_result.unwrap_err().to_string().as_str()).unwrap_err());
        }

        let username = Self::parse_decode(username_result);
        let password = Self::parse_decode(password_result);

        if username.is_err() {
            return Err(Self::err_decode(username.unwrap_err().to_string().as_str()).unwrap_err());
        }

        if password.is_err() {
            return Err(Self::err_decode(password.unwrap_err().to_string().as_str()).unwrap_err());
        }

        return Ok(UsernamePassword { username: username.unwrap(), password: password.unwrap() });
    }

    fn parse_decode(username_result: Result<Vec<u8>, DecodeError>) -> Result<String, AuthenticationConversionError> {
        match String::from_utf8(username_result.unwrap()) {
            Ok(ok_result) => {
                Ok::<String, AuthenticationConversionError>(ok_result)
            }
            Err(err_result) => {
                Self::username_not_decoded_err(Some(err_result.to_string()))
            }
        }
    }

    fn password_not_decoded_err(err_message: Option<String>) -> Result<String, AuthenticationConversionError> {
        err_message.map(|err| Self::err_decode(err.as_str()))
            .or(Some(Self::err_decode("Error decoding password")))
            .unwrap()
    }

    fn username_not_decoded_err(err_message: Option<String>) -> Result<String, AuthenticationConversionError> {
        err_message.map(|err| Self::err_decode(err.as_str()))
            .or(Some(Self::err_decode("Error decoding username")))
            .unwrap()
    }

    fn err_decode(error_message: &str) -> Result<String, AuthenticationConversionError> {
        Err(AuthenticationConversionError::new(error_message.to_string()))
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

