use std::ops::DerefMut;

#[cfg(test)]
mod test_security {
    use std::any::Any;
    use std::ops::DerefMut;
    use std::sync::{Arc, Mutex};
    use serde::{Deserialize, Serialize};
    use knockoff_security::knockoff_security::authentication_type::{Anonymous, AuthenticationConversionError, GrantedAuthority, JwtToken, UsernamePassword};
    use module_macro_lib::{AuthenticationType, AuthenticationTypeConverter};
    use web_framework_shared::convert::Converter;
    use crate::web_framework::filter::filter::DelegatingFilterProxy;
    use crate::web_framework::request::request::WebResponse;
    use crate::web_framework::security::security_filter::{AuthenticationFilter, UsernamePasswordAuthenticationFilter};
    use web_framework_shared::request::WebRequest;
    use crate::web_framework::context_builder::{AuthenticationConverterRegistryBuilder, DelegatingAuthenticationManagerBuilder};
    use crate::web_framework::convert::Registration;
    use crate::web_framework::security::authentication::{Authentication, AuthenticationConverter, AuthenticationProvider, AuthenticationToken, DelegatingAuthenticationManager};
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

    #[test]
    fn test_delegating_authentication_manager() {

        pub struct TestAuthProvider;
        impl AuthenticationProvider for TestAuthProvider {
            fn supports(&self, authentication_token: &AuthenticationType) -> bool {
                true
            }

            fn authenticate(&self, auth_token: &mut AuthenticationToken) -> AuthenticationToken {
                auth_token.authenticated = true;
                auth_token.to_owned()
            }
        }

        let mut d = DelegatingAuthenticationManagerBuilder::new();
        d.register(Box::new(TestAuthProvider{}));
        let d = d.build();
        let out = d.authenticate(&mut AuthenticationToken::default());
        assert!(out.authenticated);
    }

    #[test]
    fn test_authentication_converter_builder() {
        pub struct TestAuthConverter;
        impl Converter<WebRequest, Result<Authentication, AuthenticationConversionError>> for TestAuthConverter {
            fn convert(&self, from: &WebRequest) -> Result<Authentication, AuthenticationConversionError> {
                Ok(Authentication::default())
            }
        }
        impl AuthenticationConverter for TestAuthConverter {
        }

        pub struct TestAuthTypeConverter;
        impl Converter<WebRequest, Result<AuthenticationType, AuthenticationConversionError>> for TestAuthTypeConverter {
            fn convert(&self, from: &WebRequest) -> Result<AuthenticationType, AuthenticationConversionError> {
                Ok(AuthenticationType::Unauthenticated(Anonymous::default()))
            }
        }
        impl AuthenticationTypeConverter for TestAuthTypeConverter {
        }

        let auth_registry = AuthenticationConverterRegistryBuilder::new();
        auth_registry.register_authentication_converter(TestAuthConverter{});
        auth_registry.register_authentication_type_converter(TestAuthTypeConverter{});
        let auth_registry_built = auth_registry.build();

        let auth = auth_registry_built.convert(&WebRequest::default());

        assert!(auth.is_ok());
    }


}

