use std::marker::PhantomData;
use lazy_static::lazy_static;
use hyper::{HyperRequestConverter, HyperRequestStream};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use web_framework::{create_message_converter, default_message_converters};
use web_framework::web_framework::convert::{ConverterRegistryBuilder, EndpointRequestExtractor, MessageConverter, Registration};
use web_framework::web_framework::dispatch::Dispatcher;
use web_framework::web_framework::filter::filter::{Action, FilterChain, RequestResponseActionFilter};
use web_framework::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
use web_framework::web_framework::security::security::{AuthenticationConverterRegistryBuilder, AuthenticationProvider, AuthenticationToken, AuthenticationType, AuthenticationTypeConverterImpl, DelegatingAuthenticationManagerBuilder, UsernamePasswordAuthenticationProvider};
use web_framework::web_framework::http::{RequestExecutorImpl};
use web_framework::web_framework::context::{ApplicationContext, ApplicationContextBuilder, FilterRegistrar, RequestContext, RequestContextBuilder};
use web_framework::web_framework::message::MessageType;

#[tokio::main]
async fn main() {
    let filter: RequestResponseActionFilter<Example, Example> = RequestResponseActionFilter::new(
        Box::new(TestAction::default()), None
    );
    let mut filter_registrar = FilterRegistrar {
        filters: Arc::new(Mutex::new(vec![])),
        build: false,
        fiter_chain: Arc::new(FilterChain::default())
    };

    default_message_converters!();
    create_message_converter!(
        (crate::NewConverter1 => NewConverter1{} =>> "custom/convert1" => NewConverter1 => new_converter_1)
        ===> Example => DelegatingMessageConverter
    );

    create_message_converter!(
        (crate::NewConverter3 => NewConverter3{} =>> "custom/convert1" => NewConverter3 => new_converter)
        ===> Example1 => ExampleDelegatingMessageConverter
    );

    filter_registrar.register(filter);
    let ctx_builder = ApplicationContextBuilder::<Example, Example> {
        filter_registry: Some(Arc::new(Mutex::new(filter_registrar))),
        request_context_builder: Some(Arc::new(Mutex::new(RequestContextBuilder {
            message_converter_builder: ConverterRegistryBuilder {
                converters: Arc::new(Mutex::new(Some(Box::new(DelegatingMessageConverter::new())))),
                request_convert: Arc::new(Mutex::new(Some(Box::new(EndpointRequestExtractor{}))))
            },
            authentication_manager_builder: DelegatingAuthenticationManagerBuilder {
                providers: Arc::new(Mutex::new(vec![].into())),
            },
        }))),
        authentication_converters: Some(Arc::new(AuthenticationConverterRegistryBuilder {
            converters: Arc::new(Mutex::new(vec![])),
            authentication_type_converter: Arc::new(Mutex::new(&AuthenticationTypeConverterImpl{}))
        })),
    };
    let mut r: HyperRequestStream<Example, Example> = HyperRequestStream::new(
        RequestExecutorImpl {
            ctx: ctx_builder.build()
        }
    );

    r.do_run();

    let filter1: RequestResponseActionFilter<Example1, Example1> = RequestResponseActionFilter::new(
        Box::new(TestAction::default()), None
    );
    let mut filter_registrar1 = FilterRegistrar {
        filters: Arc::new(Mutex::new(vec![])),
        build: false,
        fiter_chain: Arc::new(FilterChain::default())
    };
    filter_registrar1.register(filter1);
    let ctx_builder1 = ApplicationContextBuilder::<Example1, Example1> {
        filter_registry: Some(Arc::new(Mutex::new(filter_registrar1))),
        request_context_builder: Some(Arc::new(Mutex::new(RequestContextBuilder {
            message_converter_builder: ConverterRegistryBuilder {
                converters: Arc::new(Mutex::new(Some(Box::new(ExampleDelegatingMessageConverter::new())))),
                request_convert: Arc::new(Mutex::new(Some(Box::new(EndpointRequestExtractor{}))))
            },
            authentication_manager_builder: DelegatingAuthenticationManagerBuilder {
                providers: Arc::new(Mutex::new(Arc::new(vec![Box::new(UsernamePasswordAuthenticationProvider{})]))),
            },
        }))),
        authentication_converters: Some(Arc::new(AuthenticationConverterRegistryBuilder {
            converters: Arc::new(Mutex::new(vec![])),
            authentication_type_converter: Arc::new(Mutex::new(&AuthenticationTypeConverterImpl{}))
        })),
    };
    let mut r: HyperRequestStream<Example1, Example1> = HyperRequestStream::new(
        RequestExecutorImpl {
            ctx: ctx_builder1.build()
        }
    );

    r.do_run().await;

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Example {
    value: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Example1 {
    value: String
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

struct TestAction1;
impl Action<Example1, Example1> for TestAction {
    fn do_action(
        &self,
        metadata: EndpointMetadata,
        request: &Option<Example1>,
        web_request: &WebRequest,
        response: &mut WebResponse,
        context: &RequestContext<Example1, Example1>,
        ctx: &ApplicationContext<Example1, Example1>
    ) -> Option<Example1> {
        Some(Example1::default())
    }

    fn authentication_granted(&self, token: &Option<AuthenticationToken<AuthenticationType>>) -> bool {
        true
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        true
    }

    fn clone(&self) -> Box<dyn Action<Example1, Example1>> {
        Box::new(TestAction::default())
    }
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
        ctx: &ApplicationContext<Example, Example>
    ) -> Option<Example> {
        Some(Example::default())
    }

    fn authentication_granted(&self, token: &Option<AuthenticationToken<AuthenticationType>>) -> bool {
        true
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        true
    }

    fn clone(&self) -> Box<dyn Action<Example, Example>> {
        Box::new(TestAction::default())
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

#[derive(Clone)]
pub struct NewConverter {

}

impl <Request, Response> MessageConverter<Request, Response> for NewConverter
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    fn new() -> Self where Self: Sized {
        todo!()
    }

    fn convert_to(&self, request: &WebRequest) -> Option<MessageType<Request>> {
        todo!()
    }

    fn convert_from(&self, request_body: &Response, request: &WebRequest) -> Option<String> {
        todo!()
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        todo!()
    }

    fn message_type(&self) -> Vec<String> {
        vec!["".to_string()]
    }
}

#[derive(Clone)]
pub struct NewConverter1 {
}

impl MessageConverter<Example, Example> for NewConverter1
{
    fn new() -> Self where Self: Sized {
        todo!()
    }

    fn convert_to(&self, request: &WebRequest) -> Option<MessageType<Example>> {
        todo!()
    }

    fn convert_from(&self, request_body: &Example, request: &WebRequest) -> Option<String> {
        todo!()
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        todo!()
    }

    fn message_type(&self) -> Vec<String> {
        vec!["".to_string()]
    }
}

#[derive(Clone)]
pub struct NewConverter3;
impl MessageConverter<Example1, Example1> for NewConverter3 {
    fn new() -> Self where Self: Sized {
        Self {}
    }

    fn convert_to(&self, request: &WebRequest) -> Option<MessageType<Example1>> {
        todo!()
    }

    fn convert_from(&self, request_body: &Example1, request: &WebRequest) -> Option<String> {
        todo!()
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        todo!()
    }

    fn message_type(&self) -> Vec<String> {
        todo!()
    }
}
