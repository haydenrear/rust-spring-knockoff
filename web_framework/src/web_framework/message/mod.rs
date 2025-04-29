use serde::{Deserialize, Serialize};
use web_framework_shared::authority::GrantedAuthority;
use web_framework_shared::dispatch_server::Handler;
use web_framework_shared::request::{EndpointMetadata, WebRequest, WebResponse};
use crate::web_framework::context::{Context, RequestContextData, UserRequestContext};
use crate::web_framework::convert::{Converters, RequestTypeExtractor};
use crate::web_framework::request_context::SessionContext;
use crate::web_framework::security::security_filter::UsernamePasswordAuthenticationFilter;

#[derive(Copy, Clone, Serialize)]
pub struct MessageType<T: Serialize>
    where Self: 'static, T: 'static
{
    pub message: Option<T>,
}

pub struct MessageConverterFilter;

impl <Request, Response> Handler<Request, Response, UserRequestContext<Request>, RequestContextData<Request, Response>>
for MessageConverterFilter
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
{
    fn do_action(
        &self,
        web_request: &WebRequest,
        response: &mut WebResponse,
        application_context: &RequestContextData<Request, Response>,
        request_context: &mut Option<Box<UserRequestContext<Request>>>
    ) -> Option<Response> {

        application_context.request_context_data
            .request_context
            .convert_extract(&web_request)
            .map(|e| request_context.as_mut()
                .map(|mut r| r.endpoint_metadata = Some(e)));

        application_context
            .request_context_data
            .request_context
            .convert_to(&web_request)
            .map(|converted| {
                println!("Found to convert!");
                request_context
                    .as_mut()
                    .map(|r| r.request = converted.message)
            });

        None
    }

    fn authentication_granted(&self, token: &Option<Box<UserRequestContext<Request>>>) -> bool {
        true
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        true
    }

}
