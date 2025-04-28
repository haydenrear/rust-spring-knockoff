#[cfg(test)]
mod test_filter {
    use crate::web_framework::context::{Context, RequestContextData, UserRequestContext};
    use crate::web_framework::convert::ConverterRegistry;
    use crate::web_framework::filter::filter::{Filter, FilterChain};
    use crate::web_framework::message::{MessageConverterFilter, MessageType};
    use crate::{create_message_converter, default_message_converters};
    use circular::Buffer;
    use serde::{Deserialize, Serialize};
    use std::io::{Read, Write};
    use std::sync::Arc;
    use web_framework_shared::dispatch_server::Handler;
    use web_framework_shared::request::WebResponse;
    use web_framework_shared::request::{EndpointMetadata, WebRequest};


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
    impl Handler<Example, Example, UserRequestContext<Example>, RequestContextData<Example, Example>> for TestAction {
        fn do_action(&self, web_request: &WebRequest, response: &mut WebResponse, context: &RequestContextData<Example, Example>, request_context: &mut Option<UserRequestContext<Example>>) -> Option<Example> {
            Some(Example{ value: String::from("hello!") })
        }

        fn authentication_granted(&self, token: &Option<UserRequestContext<Example>>) -> bool {
            true
        }

        fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
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

    default_message_converters!();

    #[test]
    fn test_filter() {
        use web_framework::convert::MessageConverter;
        create_message_converter!(
            (JsonMessageConverter => JsonMessageConverter{} =>> "application/json" => JsonMessageConverter => json_message_converter)
            ===> Example => DelegatingMessageConverter);

        let mapping = Filter {
            actions: Arc::new(MessageConverterFilter{}),
            dispatcher: Default::default(),
            order: 0,
        };
        let one = Filter {
            actions: Arc::new((TestAction {})),
            dispatcher: Default::default(),
            order: 1,
        };
        let mut fc = FilterChain::new(vec![one, mapping]);
        let x = &mut WebResponse::default();
        let ctx = Context::with_converter_registry(
            ConverterRegistry::new(None,
                                   Some(Box::new(DelegatingMessageConverter::new()))));
        let mut request = WebRequest::default();
        request.headers.insert("Content-Type".into(), "application/json".into());
        fc.do_filter(&request, x, &RequestContextData{ request_context_data: ctx },
                     &mut Some(UserRequestContext::default().into()));

        assert_eq!(x.response, serde_json::to_string(&Example{value: "hello!".to_string() }).unwrap());
    }

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
