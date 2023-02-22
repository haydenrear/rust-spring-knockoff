use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::web_framework::context::{ApplicationContext, RequestContext};
use crate::web_framework::filter::filter::Action;
use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
use crate::web_framework::security::security::{Authentication, AuthenticationConversionError, AuthenticationConverter, AuthenticationToken, AuthenticationTypeConverter, AuthenticationTypeConverterImpl, Converter};

pub mod security_filter {
    use crate::web_framework::context::ApplicationContext;
    use crate::web_framework::filter::filter::FilterChain;
    use crate::web_framework::request::request::{WebRequest, WebResponse};

    pub struct SecurityFilter;

    //TODO: replace with Action

    // impl SecurityFilter {
    //     fn filter(&self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext) {
    //         todo!()
    //     }
    //
    // }

}

//TODO: replace filter with action
pub trait AuthenticationFilter{
    fn try_convert_to_authentication(
        &self,
        request: &WebRequest,
    ) -> Result<Option<Authentication>, AuthenticationConversionError>;
}

pub struct UsernamePasswordAuthenticationFilter {
    converter: Arc<Box<dyn AuthenticationTypeConverter>>
}

impl Default for UsernamePasswordAuthenticationFilter {
    fn default() -> Self {
        Self {
            converter: Arc::new(Box::new(AuthenticationTypeConverterImpl::new()))
        }
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

        self.converter.convert(web_request)
            .ok().map(|auth_type| {
                application_context
                    .authentication_converters
                    .converters
                    .iter()
                    .filter(|c| c.supports(&auth_type))
                    .map(|c| c.convert(&auth_type))
                    .for_each(|mut auth_token| {
                        application_context
                            .request_context
                            .authentication_manager
                            .authenticate(&mut auth_token)
                    })
            })
            .map(|f| None)
            .unwrap()

    }

    fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
        todo!()
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        todo!()
    }

    fn clone(&self) -> Box<dyn Action<Request, Response>> {
        todo!()
    }
}

impl AuthenticationFilter for UsernamePasswordAuthenticationFilter {
    fn try_convert_to_authentication(
        &self,
        request: &WebRequest,
    ) -> Result<Option<Authentication>, AuthenticationConversionError> {
        Ok(None)
    }
}
