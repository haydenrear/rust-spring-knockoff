#[cfg(test)]
mod test_filter {
    use crate::context::RequestContext;
    use crate::controller::Dispatcher;
    use crate::convert::{ConverterRegistry, Registry};
    use crate::filter::filter::{Action, Filter, FilterChain, FilterImpl, MediaType};
    use crate::message::MessageType;
    use crate::request::request::{EndpointMetadata, RequestExtractor, ResponseWriter};
    use crate::request::request::{HttpRequest, HttpResponse};
    use crate::security::security::AuthenticationToken;
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};
    use std::any::Any;
    use std::cell::RefCell;
    use std::collections::LinkedList;
    use std::io::Write;
    use std::net::TcpStream;
    use std::ops::Deref;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Example {
        value: String,
    }

    impl Default for Example {
        fn default() -> Self {
            Example {
                value: String::from("hello!"),
            }
        }
    }

    struct TestMessageConverter;

    #[derive(Serialize, Deserialize)]
    struct TestJson {
        value: String,
    }

    struct TestAction;
    impl Action<Example, Example> for TestAction {
        fn do_action(
            &self,
            metadata: EndpointMetadata,
            request: &Option<Example>,
            context: &RequestContext,
        ) -> Option<Example> {
            Some(Example::default())
        }

        fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
            true
        }
    }

    impl Clone for TestAction {
        fn clone(&self) -> Self {
            Self
        }
    }

    impl Default for TestAction {
        fn default() -> Self {
            Self
        }
    }

    #[test]
    fn test_filter() {
        let one = FilterImpl {
            actions: Box::new(TestAction::default()),
            dispatcher: Dispatcher::default(),
        };
        let mut fc = FilterChain::new(vec![&one]);
        fc.do_filter(&HttpRequest::default(), &mut HttpResponse::default());
        assert_eq!(fc.num, 0);
    }

    #[test]
    fn test_get_in_filter() {
        let one = &FilterImpl {
            actions: Box::new(TestAction::default()),
            dispatcher: Dispatcher::default(),
        };
        let mut fc = FilterChain::new(vec![one]);
        let mut request = HttpRequest::default();
        request
            .headers
            .insert("MediaType".to_string(), "json".to_string());
        request.body = serde_json::to_string(&Example::default()).unwrap();
        let mut response = HttpResponse::default();
        fc.do_filter(&request, &mut response);
        assert_eq!(fc.num, 0);
        assert_eq!(response.response, request.body)
    }

    #[test]
    fn filter_application_builder() {
        let mut vec: Vec<&dyn Filter> = vec![];
        vec.push(&FilterImpl {
            actions: Box::new(TestAction {}),
            dispatcher: Dispatcher::default(),
        });
    }

    #[test]
    fn test_registry() {
        let ctx = RequestContext::default();
        let registrations = ctx.message_converters.read_only_registrations();
        assert_eq!(registrations.len(), 2);
    }
}
