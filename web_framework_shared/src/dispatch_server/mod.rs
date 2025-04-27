use serde::{Deserialize, Serialize};
use crate::authority::GrantedAuthority;
use crate::controller::{HandlerInterceptor};
use crate::request::{EndpointMetadata, WebRequest, WebResponse};

/// dispatch server -> handler mapping -> execution chain -> request executor
/// Dispatcher server generated with a reference to all of the HandlerAdapters,
/// along with the HandlerMapping being generated with all of the implementations of the
/// HandlerMethod.
///
/// HandlerInterceptors will be the current filter chain, for getting session info, etc. and the
/// SecurityFilterChain, for loading authentication information, requiring authentication, message
/// converters, etc for the pre, and then for post any cleanup, especially for security.
///
/// So the general flow is
///
/// DispatcherServer receives the request - contains a HandlerMapping instance generated from user info
/// HandlerExecutionChain specific to the request is retrieved. The HandlerExecutionChain has the filter chains
/// provided by the framework in the pre and the post. For instance, the SecurityFilterChain is added in the
/// pre to prepare the request, as well as the session, converters, including message converter,
/// and other user provided filters are also added here.
/// - These are all provided as HandlerInterceptors, which are each passed the HandlerMethod object as well
///     as the context for the Actions that they have in their filters.
/// Then after all of the HandlerInterceptor's are finished updating the RequestContext and the WebResponse,
/// as well as extracting the Request object from the WebRequest to be passed in using the MessageConverter,
/// the HandlerAdapter is called, which is an adapter to the user provided controller method.
// pub struct DispatcherServer<T: HandlerMapping> {
//     handler_mapping: T
// }
//
// impl <T: HandlerMapping>  DispatcherServer<T> {
//     pub fn do_dispatch(&self, request: &WebRequest, response: &mut WebResponse) {
//         let f = self.handler_mapping.get_handler(request);
//     }
// }

pub trait Handler<Request, Response, RequestData: ?Sized, Ctx: ?Sized>: Send + Sync
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    fn do_action(
        &self,
        web_request: &WebRequest,
        response: &mut WebResponse,
        context: &Ctx,
        request_context: &mut Option<Box<RequestData>>
    ) -> Option<Response>;

    /**
    For method level annotations (could also be done via Aspect though).
    */
    fn authentication_granted(&self, token: &Option<Box<RequestData>>) -> bool;

    /**
    determines if it matches endpoint, http method, etc.
    */
    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool;

}
