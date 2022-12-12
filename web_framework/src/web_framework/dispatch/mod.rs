use crate::web_framework::context::{ApplicationContext, RequestContext};
use crate::web_framework::convert::{Converters, RequestExtractor};
use crate::web_framework::filter::filter::{Action, MediaType};
use crate::web_framework::message::MessageType;
use crate::web_framework::request::request::{
    EndpointMetadata, WebRequest, WebResponse, ResponseWriter,
};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Dispatcher {
    pub context: RequestContext,
}

/**
General dispatcher for web request.
*/
impl Dispatcher {
    pub(crate) fn do_request<'a, Response, Request>(
        &self,
        request: WebRequest,
        response: &mut WebResponse,
        action: &Box<dyn Action<Request, Response>>,
        application_context: &ApplicationContext<Request, Response>
    ) where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        // TODO: action is provided by user and an attribute on the action causes this
        // method to be implemented for the action created by macro
        if action.authentication_granted(&response.session.authentication_token) {
            self.context
                .convert_to(&request)
                .and_then(|found| {
                    self.context
                        .convert_extract(&request)
                        .filter(|e| action.matches(&e))
                        .and_then(|metadata| {
                            action.do_action(metadata, &found.message, &request, response, &self.context, application_context)
                        })
                        .and_then(|action_response| {
                            self.context.convert_from(&found.message, MediaType::Json)
                        })
                })
                .map(|response_to_write| {
                    response.write(response_to_write.clone().as_bytes());
                    response_to_write
                });
        }
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self {
            context: RequestContext::default(),
        }
    }
}

pub trait RequestMethodDispatcher<Response, Request> {
    fn do_method(&self) -> dyn Fn(EndpointMetadata, Request, &RequestContext) -> Response;
}

pub struct PostMethodRequestDispatcher {
    pub context: RequestContext,
}

impl Default for PostMethodRequestDispatcher {
    fn default() -> Self {
        Self {
            context: RequestContext::default(),
        }
    }
}
