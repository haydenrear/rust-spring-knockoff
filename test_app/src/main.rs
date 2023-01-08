use std::marker::PhantomData;
use lazy_static::lazy_static;
use hyper::{HyperRequestConverter, HyperRequestStream};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use web_framework::create_message_converter;
use web_framework::web_framework::convert::{ConverterRegistryBuilder, EndpointRequestExtractor, HtmlMessageConverter, JsonMessageConverter, MessageConverter, Registration};
use web_framework::web_framework::dispatch::Dispatcher;
use web_framework::web_framework::filter::filter::{Action, FilterChain, RequestResponseActionFilter};
use web_framework::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
use web_framework::web_framework::security::security::{AuthenticationConverterRegistryBuilder, AuthenticationToken, AuthenticationTypeConverterImpl, DelegatingAuthenticationManagerBuilder};
use web_framework::web_framework::http::{RequestExecutorImpl};
use web_framework::web_framework::context::{ApplicationContext, ApplicationContextBuilder, FilterRegistrar, RequestContext, RequestContextBuilder};


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
        context: &RequestContext,
        ctx: &ApplicationContext<Example, Example>
    ) -> Option<Example> {
        Some(Example::default())
    }

    fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
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

impl MessageConverter for NewConverter {
    fn do_convert(&self, request: &WebRequest) -> bool {
        todo!()
    }

    fn message_type(&self) -> Vec<String> {
        todo!()
    }
}

#[derive(Clone)]
pub struct NewConverter1 {

}

impl MessageConverter for NewConverter1 {
    fn do_convert(&self, request: &WebRequest) -> bool {
        todo!()
    }

    fn message_type(&self) -> Vec<String> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let filter: RequestResponseActionFilter<Example, Example> = RequestResponseActionFilter::new(
        Box::new(TestAction::default()), None
    );
    let mut filter_registrar = FilterRegistrar {
        filters: Arc::new(Mutex::new(vec![])),
        build: false,
        filters_build: Arc::new(FilterChain::default())
    };
    create_message_converter!(
        (crate => JsonMessageConverter{} =>> "application/json" => JsonMessageConverter => json_message_converter),
        (crate => HtmlMessageConverter{} =>> "text/html" => HtmlMessageConverter => html_message_converter),
        (crate => NewConverter{} =>> "custom/convert" => NewConverter => new_converter),
        (crate => NewConverter1{} =>> "custom/convert1" => NewConverter1 => new_converter_1)
    );
    filter_registrar.register(filter);
    let ctx_builder = ApplicationContextBuilder {
        filter_registry: Some(Arc::new(Mutex::new(filter_registrar))),
        request_context_builder: Some(Arc::new(Mutex::new(RequestContextBuilder {
            message_converter_builder: ConverterRegistryBuilder {
                converters: Arc::new(Mutex::new(Some(Box::new(DelegatingMessageConverter::new())))),
                request_convert: Arc::new(Mutex::new(Some(&EndpointRequestExtractor{})))
            },
            authentication_manager_builder: DelegatingAuthenticationManagerBuilder {
                providers: Arc::new(Mutex::new(vec![])),
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
    r.do_run().await;
}