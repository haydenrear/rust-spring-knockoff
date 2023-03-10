use crate::web_framework::context::{ApplicationContext, RequestContext};
use crate::web_framework::convert::{Converters, RequestExtractor};
use crate::web_framework::filter::filter::{Action, MediaType};
use crate::web_framework::message::MessageType;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use crate::web_framework::context_builder::RequestContextBuilder;
use crate::web_framework::request::request::{ResponseWriter, WebResponse};
use web_framework_shared::request::WebRequest;

#[derive(Clone)]
pub struct Dispatcher {
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
        if action.authentication_granted(&response.session.security_context_holder.auth_token) {
            application_context
                .request_context
                .convert_to(&request)
                .and_then(|found| {
                    application_context.request_context
                        .convert_extract(&request)
                        .filter(|e| action.matches(&e))
                        .and_then(|metadata| {
                            action.do_action(
                                metadata, &found.message,
                                &request, response,
                                &application_context.request_context,
                                application_context
                            )
                        })
                        .and_then(|action_response| {
                            let media_type = request.headers.get("mediatype").cloned()
                                .or(request.headers.get("MediaType").cloned())
                                .or(request.headers.get("Mediatype").cloned())
                                .or(Some("application/json".to_string()));

                            application_context.request_context
                                .convert_from(&action_response, &request, media_type)
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
        }
    }
}

