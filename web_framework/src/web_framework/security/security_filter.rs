use std::ptr::write_bytes;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use knockoff_security::knockoff_security::authentication_type::AuthenticationConversionError;
use authentication_gen::{AuthenticationTypeConverter, AuthenticationTypeConverterImpl};
use web_framework_shared::authority::GrantedAuthority;
use web_framework_shared::controller::{HandlerInterceptor, HandlerMethod};
use web_framework_shared::convert::Converter;
use web_framework_shared::dispatch_server::Handler;
use crate::web_framework::context::{Context, RequestContextData, RequestHelpers, UserRequestContext};
use crate::web_framework::filter::filter::{Filter, FilterChain};
use web_framework_shared::request::{ResponseBytesBuffer, WebResponse};
use web_framework_shared::request::{EndpointMetadata, WebRequest};
use crate::web_framework::convert::AuthenticationConverterRegistry;
use crate::web_framework::dispatch::FilterExecutor;
use crate::web_framework::request_context::SessionContext;
use crate::web_framework::security::authentication::{AuthenticationConverter, AuthenticationProvider, AuthenticationToken, DelegatingAuthenticationManager};
use crate::web_framework::session::session::HttpSession;

pub struct SecurityFilterChain<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    pub(crate) filters: Arc<FilterChain<Request, Response>>,
}

impl <Request, Response> SecurityFilterChain<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    pub fn do_filter(&self,
                     request: &WebRequest,
                     response: &mut WebResponse,
                     ctx: &RequestContextData<Request, Response>,
                     request_context: &mut Option<Box<UserRequestContext<Request>>>)
    {
        self.filters.do_filter(request, response, ctx, request_context);
    }
}

//TODO: replace filter with action
pub trait AuthenticationFilter
{
    fn try_convert_to_authentication(
        &self,
        request: &WebRequest,
    ) -> Result<AuthenticationToken, AuthenticationConversionError>;
}

pub struct UsernamePasswordAuthenticationFilter
{
    converter: Arc<AuthenticationConverterRegistry>,
    authentication_manager: Arc<DelegatingAuthenticationManager>
}

impl Default for UsernamePasswordAuthenticationFilter
{
    fn default() -> Self {
        Self {
            converter: Arc::new(AuthenticationConverterRegistry::new()),
            authentication_manager: Arc::new(DelegatingAuthenticationManager::new()),
        }
    }
}

impl AuthenticationFilter for UsernamePasswordAuthenticationFilter {
    fn try_convert_to_authentication(&self, request: &WebRequest) -> Result<AuthenticationToken, AuthenticationConversionError> {
        self.converter
            .convert(request)
            .map(|mut auth_token|  self.authentication_manager.authenticate(&mut auth_token.credentials) )
    }
}

impl UsernamePasswordAuthenticationFilter
{
    pub fn username_password_filter<Request, Response>(
        converter: Arc<AuthenticationConverterRegistry>,
        authentication_manager: Arc<DelegatingAuthenticationManager>,
        dispatcher: Arc<FilterExecutor>,
        order: Option<u8>
    ) -> Filter<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        Filter::new(
            Arc::new(
                Self { converter, authentication_manager }
            ),
            order,
            dispatcher
        )
    }
}

impl <Request, Response> Handler<Request, Response, UserRequestContext<Request>, RequestContextData<Request, Response>> for UsernamePasswordAuthenticationFilter
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

        self.try_convert_to_authentication(web_request)
            .map(|auth| {
                request_context.as_mut().map(|mut request_context| {
                    request_context.request_context.http_session.security_context_holder.auth_token = Some(auth.to_owned());
                });
                auth
            })
            .expect("Panic experienced while authenticating user.");

        None
    }

    fn authentication_granted(&self, token: &Option<Box<UserRequestContext<Request>>>) -> bool {
        true
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        todo!()
    }

}
