use std::sync::Arc;
use crate::web_framework::context::{Context, RequestHelpers};
use crate::web_framework::convert::{Converters, RequestExtractor};
use crate::web_framework::filter::filter::MediaType;
use crate::web_framework::message::MessageType;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use web_framework_shared::dispatch_server::Handler;
use crate::web_framework::context_builder::RequestContextBuilder;
use web_framework_shared::request::{ResponseWriter, WebRequest, WebResponse};
use crate::web_framework::request_context::RequestContext;
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
        action: Arc<dyn Handler<Request, Response, RequestContext, Context<Request, Response>>>,
        application_context: &Context<Request, Response>,
        request_context: &mut RequestContext,
    ) where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        if action.authentication_granted(
            request_context.http_session.security_context_holder.auth_token
                    .as_ref()
                    .map(|a| &a.authorities)
                    .or(Some(&vec![]))
                    .unwrap()
        )
        {
            application_context
                .request_context
                .convert_to(&request)
                .and_then(|found| {
                    application_context.request_context
                        .convert_extract(&request)
                        .filter(|e| action.matches(&e))
                        .and_then(|_| {
                            action.do_action(
                                 &found.message,
                                &request,
                                 response,
                                application_context,
                                request_context
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

impl Default for FilterExecutor {
    fn default() -> Self {
        Self {
        }
    }
}

