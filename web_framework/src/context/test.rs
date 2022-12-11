#[cfg(test)]
mod test {
    use std::collections::LinkedList;
    use crate::context::{ApplicationContext, RequestContext};
    use crate::convert::{JsonMessageConverter, MessageConverter, Registration};
    use crate::filter::filter::{Filter, FilterChain};
    use crate::request::request::{WebRequest, WebResponse};
    use crate::security::security::{AuthenticationAware, AuthenticationConverter, AuthenticationToken, AuthenticationType, Authority, AuthType, Converter, UsernamePassword, UsernamePasswordAuthenticationConverter};

    pub struct TestFilter;

    impl <'a> Filter for TestFilter
    where
        'a: 'static
    {
        fn filter(&self, request: &WebRequest, response: &mut WebResponse, filter: FilterChain, ctx: &ApplicationContext) {
        }
    }

    pub struct TestUsernamePasswordAuthenticationConverter;


    impl Converter<AuthenticationType, AuthenticationToken> for TestUsernamePasswordAuthenticationConverter {
        fn convert(&self, from: &AuthenticationType) -> AuthenticationToken {
            todo!()
        }

    }

    impl AuthenticationConverter for TestUsernamePasswordAuthenticationConverter {
        fn supports(&self, auth_type: AuthenticationType) -> bool {
            todo!()
        }
    }

    impl UsernamePasswordAuthenticationConverter for TestUsernamePasswordAuthenticationConverter {

    }

    #[test]
    fn test_building_context() {
        let mut ctx = ApplicationContext::new();
        ctx.register(&TestFilter{} as &dyn Filter);
        ctx.register(&JsonMessageConverter{} as &dyn MessageConverter);
        let username = "".to_string();
        let password = "".to_string();
        ctx.register(&TestUsernamePasswordAuthenticationConverter{} as &dyn AuthenticationConverter)
    }
}