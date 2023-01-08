#[cfg(test)]
mod test {
    use std::collections::LinkedList;
    use crate::web_framework::filter::filter::{Action, RequestResponseActionFilter};
    use crate::web_framework::context::{ApplicationContext, RequestContext};
    use crate::web_framework::convert::{JsonMessageConverter, MessageConverter, Registration};
    use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
    use crate::web_framework::security::security::{AuthenticationAware, AuthenticationConverter, AuthenticationToken, AuthenticationType, Authority, AuthType, Converter, UsernamePassword, UsernamePasswordAuthenticationConverter};
    use serde::{Serialize, Deserialize};

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

    impl UsernamePasswordAuthenticationConverter for TestUsernamePasswordAuthenticationConverter {}


    pub struct TestAction {}

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct Example {
        value: String,
    }

    impl Action<Example, Example> for TestAction {
        fn do_action(&self, metadata: EndpointMetadata, request: &Option<Example>, web_request: &WebRequest, response: &mut WebResponse, context: &RequestContext, application_context: &ApplicationContext<Example, Example>) -> Option<Example> {
            todo!()
        }

        fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
            todo!()
        }

        fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
            todo!()
        }

        fn clone(&self) -> Box<dyn Action<Example, Example>> {
            todo!()
        }
    }

    #[test]
    fn test_building_context() {
        let mut ctx = ApplicationContext::new();
        ctx.filter_registry.register(RequestResponseActionFilter::new(Box::new(TestAction {}), Some(0)));
        ctx.register(&JsonMessageConverter{} as &dyn MessageConverter);
        let username = "".to_string();
        let password = "".to_string();
        ctx.register(&TestUsernamePasswordAuthenticationConverter{} as &dyn AuthenticationConverter)
    }
}