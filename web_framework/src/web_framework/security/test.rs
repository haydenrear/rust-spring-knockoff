#[cfg(test)]
mod test_security {
    use std::any::Any;
    use crate::web_framework::filter::filter::FilterChain;
    use crate::web_framework::request::request::{WebRequest, WebResponse};
    use crate::web_framework::security::security::{AuthenticationType, JwtToken, UsernamePassword};
    use crate::web_framework::security::security_filter::{AuthenticationFilter, UsernamePasswordAuthenticationFilter};

    #[test]
    fn test_split() {
        let username_password_auth_filter = UsernamePasswordAuthenticationFilter::default();
        let mut request = WebRequest::default();
        request.headers.insert(
            String::from("Authorization"),
            String::from("faslkjaf:as;dljfkas"),
        );
        username_password_auth_filter.try_convert_to_authentication(&request);
    }

    #[test]
    fn test_auth_type() {
        let username = "".to_string();
        let password = "".to_string();
        let username_type_id = AuthenticationType::Password(UsernamePassword{username, password})
            .type_id();
        let jwt_type_id = AuthenticationType::Jwt(JwtToken{ token: "".to_string() })
            .type_id();
        assert_ne!(username_type_id, jwt_type_id)
    }
}
