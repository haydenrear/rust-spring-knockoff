#[cfg(test)]
mod test_filter {
    use crate::web_framework::context::{ApplicationContext, RequestContext};
    use crate::web_framework::dispatch::Dispatcher;
    use crate::web_framework::convert::{ConverterRegistry, Registry};
    use crate::web_framework::filter::filter::{Action, FilterChain, RequestResponseActionFilter, MediaType};
    use crate::web_framework::message::MessageType;
    use crate::web_framework::request::request::{EndpointMetadata, ResponseBytesBuffer, ResponseWriter};
    use crate::web_framework::request::request::{WebRequest, WebResponse};
    use crate::web_framework::security::security::{AuthenticationToken, AuthenticationType, AuthType};
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};
    use std::any::Any;
    use std::cell::RefCell;
    use std::collections::LinkedList;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::ops::Deref;
    use futures::SinkExt;
    use circular::Buffer;

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
            web_request: &WebRequest,
            response: &mut WebResponse,
            context: &RequestContext<Example, Example>,
            application_context: &ApplicationContext<Example, Example>
        ) -> Option<Example> {
            Some(Example{ value: String::default() })
        }

        fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
            true
        }

        fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
            true
        }

        fn clone(&self) -> Box<dyn Action<Example, Example>> {
            todo!()
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
        let one = RequestResponseActionFilter {
            actions: Box::new((TestAction {})),
            dispatcher: Default::default(),
            order: 0,
        };
        let mut fc = FilterChain::new(vec![one]);
        fc.do_filter(&WebRequest::default(), &mut WebResponse::default(), &ApplicationContext::new());
    }

    #[test]
    fn test_get_in_filter() {
        let one = RequestResponseActionFilter {
            actions: Box::new((TestAction {})),
            dispatcher: Default::default(),
            order: 0,
        };
        let mut fc = FilterChain::new(vec![one]);
        let mut request = WebRequest::default();
        request
            .headers
            .insert("MediaType".to_string(), "json".to_string());
        request.body = serde_json::to_string(&Example::default()).unwrap();
        let mut response = WebResponse::default();
        fc.do_filter(&request, &mut response, &ApplicationContext::new());
        let response_val = String::from_utf8(response.response_bytes().unwrap())
            .unwrap();
        assert_eq!(response_val, request.body);
        assert_eq!(0, response.response_bytes().unwrap().len());
    }

    #[test]
    fn filter_application_builder() {
        let mut vec: Vec<RequestResponseActionFilter<Example, Example>> = vec![];
        vec.push(RequestResponseActionFilter {
            actions: Box::new(TestAction {}),
            dispatcher: Default::default(),
            order: 0,
        });
    }

    // #[test]
    // fn test_registry() {
    //     let ctx = RequestContext::default();
    //     let registrations = ctx.message_converters.read_only_registrations();
    //     assert_eq!(registrations.len(), 2);
    // }

    #[test]
    fn test_buffer() {
        let mut buffer = Buffer::with_capacity(100);
        let mut to_write: [u8; 155] = [0;155];
        for i in 0..100 {
            to_write[i] = i as u8;
        }
        buffer.write(to_write.as_slice());
        let mut to_read: [u8; 155] = [0;155];
        buffer.read(to_read.as_mut_slice());
        println!("{}", &to_read[10]);
        assert_eq!(to_read[10], 10);

        let mut to_read: [u8; 155] = [0;155];
        buffer.read(to_read.as_mut_slice());
        println!("{}", &to_read[10]);
        assert_ne!(to_read[10], 10);
    }
}
