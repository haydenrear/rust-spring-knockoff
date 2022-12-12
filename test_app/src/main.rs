use std::marker::PhantomData;
use lazy_static::lazy_static;
use hyper::{HyperRequestConverter, HyperRequestStream};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use web_framework::web_framework::convert::Registration;
use web_framework::web_framework::dispatch::Dispatcher;
use web_framework::web_framework::filter::filter::{Action, FilterChain, RequestResponseActionFilter};
use web_framework::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
use web_framework::web_framework::security::security::AuthenticationToken;
use web_framework::web_framework::http::{RequestExecutorImpl};
use web_framework::web_framework::context::{ApplicationContext, FilterRegistrar, RequestContext};


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
    filter_registrar.with_filter(filter);
    let mut r: HyperRequestStream<Example, Example> = HyperRequestStream::new(
        RequestExecutorImpl {
            ctx: ApplicationContext::with_filter_registry(filter_registrar)
        }
    );
    r.do_run().await;
}