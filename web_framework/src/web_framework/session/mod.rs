pub mod repo;
#[cfg(test)]
pub mod test;
pub mod session {

    extern crate alloc;
    extern crate core;

    use crate::web_framework::context::{RequestContextData, UserRequestContext};
    use crate::web_framework::security::authentication::AuthenticationToken;
    use crate::web_framework::security::security_context_holder::SecurityContextHolder;
    use alloc::string::String;
    use core::borrow::Borrow;
    use data_framework::{Entity, Repo};
    use futures::executor;
    use knockoff_security::knockoff_security::user_request_account::SessionData;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::any::Any;
    use std::marker::PhantomData;
    use web_framework_shared::dispatch_server::Handler;
    use web_framework_shared::request::WebResponse;
    use web_framework_shared::request::{EndpointMetadata, WebRequest};


    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct HttpSession {
        pub session_data: SessionData,
        pub security_context_holder: SecurityContextHolder,
        pub id: Option<String>,
    }

    impl HttpSession {
        pub fn new(
            id: String,
            authentication_token: Option<AuthenticationToken>,
            session_data: SessionData,
        ) -> HttpSession {
            Self {
                session_data,
                security_context_holder: SecurityContextHolder {
                    auth_token: authentication_token
                },
                id: Some(id),
            }
        }
    }

    impl Entity<String> for HttpSession {
        fn get_id(&self) -> Option<String> {
            self.id.clone()
        }
        fn set_id(&mut self, id: String) {
            self.id = Some(id);
        }
    }

    pub struct SessionFilter<'a, R>
    where
        R: Repo<'a, HttpSession, String>,
    {
        p: &'a PhantomData<dyn Any + Send + Sync>,
        repo: Box<R>,
    }

    impl<'a, R, Request, Response> Handler<Request, Response, UserRequestContext<Request>, RequestContextData<Request, Response>> for SessionFilter<'a, R>
    where
        R: Repo<'a, HttpSession, String>,
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {

        fn do_action(
            &self,
            mut web_request: &WebRequest,
            mut response: &mut WebResponse,
            application_context: &RequestContextData<Request, Response>,
            request_context: &mut Option<Box<UserRequestContext<Request>>>
        ) -> Option<Response> {
            if let Some(session) = web_request
                .headers
                .get("R_SESSION_ID")
                .and_then(|session_id| executor::block_on(self.repo.find_by_id(session_id))) {
                    request_context.as_mut().map(|mut request_context| {
                        request_context.request_context.http_session = session;
                    });
            }
            None
        }

        fn authentication_granted(&self, token: &Option<Box<UserRequestContext<Request>>>) -> bool {
            true
        }

        fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
            true
        }

    }
}
