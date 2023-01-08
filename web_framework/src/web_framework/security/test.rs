#[cfg(test)]
mod test_security {
    use crate::web_framework::filter::filter::{FilterChain};
    use crate::web_framework::request::request::{WebRequest, WebResponse};
    use crate::web_framework::security::security::{AuthenticationFilter, UsernamePasswordAuthenticationFilter};

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
}
