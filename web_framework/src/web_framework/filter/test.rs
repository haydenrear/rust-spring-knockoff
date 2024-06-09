#[cfg(test)]
mod test_filter {
    use crate::web_framework::context::{Context, RequestHelpers};
    use crate::web_framework::dispatch::FilterExecutor;
    use crate::web_framework::convert::{ConverterRegistry, Registry};
    use crate::web_framework::filter::filter::{Filter, FilterChain, MediaType};
    use crate::web_framework::message::MessageType;
    use crate::web_framework::security::authentication::AuthenticationToken;
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
    use authentication_gen::AuthenticationType;
    use web_framework_shared::dispatch_server::Handler;
    use web_framework_shared::request::{EndpointMetadata, WebRequest};
    use web_framework_shared::request::WebResponse;
    use crate::web_framework::session::session::HttpSession;


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
    // impl Handler<Example, Example> for TestAction {
    //     fn do_action(
    //         &self,
    //         metadata: EndpointMetadata,
    //         request: &Option<Example>,
    //         web_request: &WebRequest,
    //         response: &mut WebResponse,
    //         context: &RequestHelpers<Example, Example>,
    //         application_context: &Context<Example, Example>
    //     ) -> Option<Example> {
    //         Some(Example{ value: String::default() })
    //     }
    //
    //     fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
    //         true
    //     }
    //
    //     fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
    //         true
    //     }
    //
    //     fn clone(&self) -> Box<dyn Handler<Example, Example>> {
    //         todo!()
    //     }
    // }

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
        // let one = Filter {
        //     // actions: Box::new((TestAction {})),
        //     dispatcher: Default::default(),
        //     order: 0,
        // };
        // let mut fc = FilterChain::new(vec![one]);
        // fc.do_filter(&WebRequest::default(), &mut WebResponse::default(), &Context::new());
    }

    #[test]
    fn test_get_in_filter() {
        // let one = Filter {
        //     actions: Box::new((TestAction {})),
        //     dispatcher: Default::default(),
        //     order: 0,
        // };
        // let mut fc = FilterChain::new(vec![one]);
        let mut request = WebRequest::default();
        request
            .headers
            .insert("MediaType".to_string(), "json".to_string());
        request.body = serde_json::to_string(&Example::default()).unwrap();
        let mut response = WebResponse::default();
        // fc.do_filter(&request, &mut response, &Context::new());
        let response_val = String::from_utf8(response.response_bytes().unwrap())
            .unwrap();
        assert_eq!(response_val, request.body);
        assert_eq!(0, response.response_bytes().unwrap().len());
    }

    #[test]
    fn filter_application_builder() {
        let mut vec: Vec<Filter<Example, Example>> = vec![];
        // vec.push(Filter {
        //     actions: Box::new(TestAction {}),
        //     dispatcher: Default::default(),
        //     order: 0,
        // });
    }

    // #[test_mod]
    // fn test_registry() {
    //     let ctx = RequestContext::default_impls();
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
