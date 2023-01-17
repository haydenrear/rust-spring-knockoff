#[cfg(test)]
mod test {
    use std::collections::LinkedList;
    use std::ops::Deref;
    use std::sync::{Arc, Mutex};
    use crate::web_framework::filter::filter::{Action, RequestResponseActionFilter};
    use crate::web_framework::context::{ApplicationContext, ApplicationContextBuilder, FilterRegistrar, RequestContext, RequestContextBuilder};
    use crate::web_framework::convert::{ConverterRegistryBuilder, JsonMessageConverter, MessageConverter, Registration};
    use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
    use crate::web_framework::security::security::{AuthenticationAware, AuthenticationConverter, AuthenticationToken, AuthenticationType, Authority, AuthType, Converter, DelegatingAuthenticationManagerBuilder, UsernamePassword, UsernamePasswordAuthenticationConverter};
    use serde::{Serialize, Deserialize};

    pub struct TestUsernamePasswordAuthenticationConverter;


    impl Converter<AuthenticationType, AuthenticationToken<AuthenticationType>> for TestUsernamePasswordAuthenticationConverter {
        fn convert(&self, from: &AuthenticationType) -> AuthenticationToken<AuthenticationType> {
            todo!()
        }

    }

    impl AuthenticationConverter for TestUsernamePasswordAuthenticationConverter {
        fn supports(&self, auth_type: &AuthenticationType) -> bool {
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
        fn do_action(&self, metadata: EndpointMetadata, request: &Option<Example>, web_request: &WebRequest, response: &mut WebResponse,
                     context: &RequestContext<Example, Example>, application_context: &ApplicationContext<Example, Example>) -> Option<Example> {
            todo!()
        }

        fn authentication_granted(&self, token: &Option<AuthenticationToken<AuthenticationType>>) -> bool {
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
        let mut ctx = ApplicationContextBuilder{
            filter_registry: Some(Arc::new(Mutex::new(FilterRegistrar::new()))),
            request_context_builder: Some(Arc::new(Mutex::new(RequestContextBuilder{
                message_converter_builder: ConverterRegistryBuilder {
                    converters: Arc::new(Mutex::new(None)),
                    request_convert: Arc::new(Mutex::new(None)),
                },
                authentication_manager_builder: DelegatingAuthenticationManagerBuilder {
                    providers: Arc::new(Mutex::new(Arc::new(vec![])))
                },
            }))),
            authentication_converters: None,
        };
        ctx.filter_registry.as_ref().unwrap()
            .lock().unwrap()
            .register(RequestResponseActionFilter::new(Box::new(TestAction {}), Some(0)));
        let username = "".to_string();
        let password = "".to_string();
        ctx.register(&TestUsernamePasswordAuthenticationConverter{} as &dyn AuthenticationConverter)
    }
}