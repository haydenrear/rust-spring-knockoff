use std::ptr::write_bytes;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use knockoff_security::knockoff_security::authentication_type::AuthenticationConversionError;
use module_macro_lib::{AuthenticationTypeConverter, AuthenticationTypeConverterImpl};
use web_framework_shared::convert::Converter;
use crate::web_framework::context::{ApplicationContext, RequestContext};
use crate::web_framework::filter::filter::{Action, DelegatingFilterProxy, Filter};
use crate::web_framework::request::request::WebResponse;
use web_framework_shared::request::{EndpointMetadata, WebRequest};
use crate::web_framework::convert::AuthenticationConverterRegistry;
use crate::web_framework::security::authentication::{AuthenticationConverter, AuthenticationProvider, AuthenticationToken, DelegatingAuthenticationManager};

pub struct SecurityFilterChain<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    pub(crate) filters: Arc<DelegatingFilterProxy<Request, Response>>,
}

impl <Request, Response> SecurityFilterChain<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    pub fn do_filter(&self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext<Request, Response>) {
        self.filters.do_filter(request, response, ctx);
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
        order: Option<u8>
    ) -> Filter<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        Filter::new(
            Box::new(
                Self { converter, authentication_manager }
            ),
            order
        )
    }
}

impl <Request, Response> Action<Request, Response> for UsernamePasswordAuthenticationFilter
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
{
    fn do_action(
            &self,
            metadata: EndpointMetadata,
            request: &Option<Request>,
            web_request: &WebRequest,
            response: &mut WebResponse,
            context: &RequestContext<Request, Response>,
            application_context: &ApplicationContext<Request, Response>
        ) -> Option<Response> {

        self.try_convert_to_authentication(web_request)
            .map(|auth| {
                response.session.security_context_holder.auth_token = Some(auth.to_owned());
                auth
            })
            .expect("Panic experienced while authenticating user.");

        None
    }

    fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
        true
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        todo!()
    }

    fn clone(&self) -> Box<dyn Action<Request, Response>> {
        Box::new(Self {
            converter: self.converter.clone(),
            authentication_manager: self.authentication_manager.clone()
        })
    }
}
