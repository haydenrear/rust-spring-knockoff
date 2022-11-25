#[cfg(test)]
mod test_filter {

    use crate::request::request::{EndpointMetadata, RequestExtractor, ResponseWriter};
    use crate::request::request::{HttpRequest, HttpResponse};
    use crate::convert::ConverterContext;
    use std::cell::RefCell;
    use std::collections::LinkedList;
    use std::io::Write;
    use std::net::TcpStream;
    use lazy_static::lazy_static;
    use serde::{Serialize,Deserialize};
    use crate::context::Context;
    use crate::controller::{Dispatcher};
    use crate::filter::filter::{Action, Filter, FilterChain, FilterImpl, MediaType};
    use crate::message::MessageType;

    #[derive(Serialize,Deserialize,Debug,Clone)]
    pub struct Example {
        value: String
    }

    impl Default for Example {
        fn default() -> Self {
            Example {
                value: String::from("hello!")
            }
        }
    }

    struct TestMessageConverter;

    #[derive(Serialize, Deserialize)]
    struct TestJson {
        value: String
    }

    struct TestAction;
    impl Action<Example, Example> for TestAction  {
        fn do_action(&self,
                     metadata: EndpointMetadata,
                     request: &Option<Example>,
                     context: &Context
        ) -> Option<Example> {
            Some(Example::default())
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
            dispatcher: Dispatcher::default()
        };
        let mut fc = FilterChain::new(vec![&one]);
        fc.do_filter(&HttpRequest::default(), &mut HttpResponse::default());
        assert_eq!(fc.num, 0);
    }


    #[test]
    fn test_get_in_filter() {
        let one = &FilterImpl {
            actions: Box::new(TestAction::default()),
            dispatcher: Dispatcher::default()
        };
        let mut fc = FilterChain::new(vec![one]);
        let mut request = HttpRequest::default();
        request.body = serde_json::to_string(&Example::default())
            .unwrap();
        let mut response = HttpResponse::default();
        fc.do_filter(&request, &mut response);
        assert_eq!(fc.num, 0);
        assert_eq!(response.response, request.body)
    }

}