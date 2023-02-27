#[cfg(test)]
mod test_security {
    use std::any::Any;
    use serde::{Deserialize, Serialize};
    use knockoff_security::knockoff_security::authentication_type::{JwtToken, UsernamePassword};
    use module_macro_lib::AuthenticationType;
    use crate::web_framework::filter::filter::DelegatingFilterProxy;
    use crate::web_framework::request::request::WebResponse;
    use crate::web_framework::security::security_filter::{AuthenticationFilter, UsernamePasswordAuthenticationFilter};
    use web_framework_shared::request::WebRequest;
    use crate::web_framework::security::authentication::{DelegatingAuthenticationManager, GrantedAuthority};
    use crate::web_framework::security::http_security::HttpSecurity;

    #[test]
    fn test_split() {
        let username_password_auth_filter = UsernamePasswordAuthenticationFilter::default();

        let mut request = WebRequest::default();

        request.headers.insert(
            String::from("Authorization"),
            String::from("Basic faslkjaf:as;dljfkas"),
        );

        let converted = username_password_auth_filter.try_convert_to_authentication(&request);
        assert!(converted.is_ok());

        converted.iter()
            .for_each(|c| {
                match c.auth {
                    AuthenticationType::Password(_)  => {
                    }
                    _ => {
                        assert!(false);
                    }
                }
            })
    }

    #[test]
    fn test_auth_type() {
        let username = "".to_string();
        let password = "".to_string();
        let username_type_id = AuthenticationType::Password(UsernamePassword{username, password})
            .type_id();
        let jwt_type_id = AuthenticationType::Jwt( JwtToken{ token: "".to_string() } )
            .type_id();
        assert_ne!(username_type_id, jwt_type_id)
    }

    #[test]
    fn test_authority() {
        let g = GrantedAuthority { authority: "".to_string() };
        let auth = g.get_authority();
        assert_eq!("", auth)
    }

    #[test]
    fn test_http_security() {
        #[derive(Serialize, Deserialize, Clone, Default)]
        pub struct TestReq;
        pub trait TestTrain: Send + Sync {}
        impl TestTrain for TestReq {}

        let http = HttpSecurity::<TestReq, TestReq>::default();
        http.authorization_manager(DelegatingAuthenticationManager::new());
        assert!(http.authentication_manager.lock().unwrap().is_some());
    }
}
