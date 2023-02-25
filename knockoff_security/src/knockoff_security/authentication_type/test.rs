use std::io::Bytes;
use web_framework_shared::request::WebRequest;
use base64::{Engine as _, engine::general_purpose};
use crate::knockoff_security::authentication_type::{AuthHelper, UsernamePassword};

#[test]
fn test_authorization_header_split() {
    test_auth_header_split_template("123", "345", "Basic", "123:345");
}

#[test]
fn test_username_password_parse_creds() {
    let parsed = UsernamePassword::parse_credentials_inner(
        &test_web_request("123", "456"));
    if parsed.as_ref().is_err() {
        println!("Error parsing credentials: {}!", parsed.as_ref().unwrap_err().message);
    }
    assert!(parsed.as_ref().is_ok());
    assert_eq!(parsed.as_ref().unwrap().username, "123");
    assert_eq!(parsed.as_ref().unwrap().password, "456");
}

fn test_auth_header_split_template(username: &str, password: &str, header: &str, out: &str) {
    let split = AuthHelper::get_authorization_header_split(&test_web_request(username, password), Some(header.to_string()));
    assert!(split.is_some());
    assert_eq!(split.unwrap(), out);
}

fn test_web_request(username: &str, password: &str) -> WebRequest {
    let mut web_request = WebRequest::default();
    let mut username_password = "Basic ".to_string();
    username_password += general_purpose::STANDARD_NO_PAD.encode(username).as_str();
    username_password += ":";
    username_password += general_purpose::STANDARD_NO_PAD.encode(password).as_str();
    web_request.headers.insert("Authorization".to_string(), username_password.to_string());
    web_request
}
