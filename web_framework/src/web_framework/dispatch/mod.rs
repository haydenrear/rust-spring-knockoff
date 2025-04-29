use std::sync::Arc;
use crate::web_framework::context::{Context, RequestContextData, RequestHelpers, UserRequestContext};
use crate::web_framework::convert::{Converters, RequestTypeExtractor};
use crate::web_framework::filter::filter::MediaType;
use crate::web_framework::message::MessageType;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use web_framework_shared::authority::GrantedAuthority;
use web_framework_shared::dispatch_server::Handler;
use crate::web_framework::context_builder::RequestContextBuilder;
use web_framework_shared::request::{ResponseWriter, WebRequest, WebResponse};
use crate::web_framework::request_context::SessionContext;
use crate::web_framework::session::session::HttpSession;

#[derive(Clone)]
pub struct FilterExecutor;

/**
General dispatch_server for web request.
*/
impl FilterExecutor {
    pub(crate) fn do_request<'a, Response, Request>(
        &self,
        request: &WebRequest,
        response: &mut WebResponse,
        action: Arc<dyn Handler<Request, Response, UserRequestContext<Request>, RequestContextData<Request, Response>>>,
        application_context: &RequestContextData<Request, Response>,
        request_context: &mut Option<Box<UserRequestContext<Request>>>,
    ) where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        if action.authentication_granted(request_context) {
            // TODO: all of this logic will be added to the execution filter chain
            action.do_action(&request, response, application_context, request_context)
                .and_then(|action_response| {
                    let media_type = request.headers.get("mediatype").cloned()
                        .or(request.headers.get("MediaType").cloned())
                        .or(request.headers.get("Mediatype").cloned())
                        .or(Some("application/json".to_string()));

                    application_context.request_context_data.request_context
                        .convert_from(&action_response, &request, media_type)
                })
                .map(|response_to_write| {
                    println!("Found response!");
                    response.write(response_to_write.clone().as_bytes());
                    response_to_write
                });
        }
    }
}

impl Default for FilterExecutor {
    fn default() -> Self {
        Self {
        }
    }
}

