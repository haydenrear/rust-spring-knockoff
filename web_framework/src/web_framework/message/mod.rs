use serde::{Deserialize, Serialize};
use web_framework_shared::authority::GrantedAuthority;
use web_framework_shared::dispatch_server::Handler;
use web_framework_shared::request::{EndpointMetadata, WebRequest, WebResponse};
use crate::web_framework::context::{Context, RequestContextData, UserRequestContext};
use crate::web_framework::request_context::RequestContext;
use crate::web_framework::security::security_filter::UsernamePasswordAuthenticationFilter;

#[derive(Copy, Clone, Serialize)]
pub struct MessageType<T: Serialize> where Self: 'static, T: 'static{
    pub message: Option<T>,
}

pub struct MessageConverterFilter;

impl <Request, Response> Handler<Request, Response, UserRequestContext<Request>, RequestContextData<Request, Response>> for MessageConverterFilter
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
{
    fn do_action(
        &self,
        request: &Option<Request>,
        web_request: &WebRequest,
        response: &mut WebResponse,
        application_context: &RequestContextData<Request, Response>,
        request_context: &mut UserRequestContext<Request>
    ) -> Option<Response> {
        None
    }

    fn authentication_granted(&self, token: &Vec<GrantedAuthority>) -> bool {
        true
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        todo!()
    }

}
