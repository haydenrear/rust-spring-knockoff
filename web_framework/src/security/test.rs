#[cfg(test)]
mod test_security {
    use crate::filter::filter::{Filter, FilterChain};
    use crate::request::request::{HttpRequest, HttpResponse};
    use crate::security::security::{AuthenticationFilter, UsernamePasswordAuthenticationFilter};

    #[test]
    fn test_split() {
        let username_password_auth_filter = UsernamePasswordAuthenticationFilter::default();
        let mut request = HttpRequest::default();
        request.headers.insert(
            String::from("Authorization"),
            String::from("faslkjaf:as;dljfkas"),
        );
        username_password_auth_filter.try_convert_to_authentication(request);
    }
}
