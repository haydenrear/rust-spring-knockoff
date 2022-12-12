use lazy_static::lazy_static;
use hyper::{HyperRequestConverter, HyperRequestStream};
use web_framework::http::{RequestExecutorImpl};
use web_framework::context::{ApplicationContext, RequestContext};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use web_framework::convert::Registration;
use web_framework::dispatch::Dispatcher;
use web_framework::filter::filter::{Action, RequestResponseActionFilter};
use web_framework::request::request::EndpointMetadata;
use web_framework::security::security::AuthenticationToken;


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

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        true
    }

    fn replicate(&self) -> Box<dyn Action<Example, Example>> {
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



#[tokio::main]
async fn main() {
    let one = RequestResponseActionFilter::new(
        Box::new(TestAction::default())
    );
    let mut r = HyperRequestStream::new();
    r.request_executor.ctx.filter_registry.register(Box::new(one));
    r.do_run().await;
}