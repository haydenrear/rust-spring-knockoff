pub mod repo;
#[cfg(test)]
pub mod test;
pub mod session {

    extern crate alloc;
    extern crate core;

    use crate::web_framework::filter::filter::{Action, DelegatingFilterProxy};
    use web_framework_shared::request::{EndpointMetadata, WebRequest};
    use alloc::string::String;
    use async_std::task as async_task;
    use core::borrow::Borrow;
    use data_framework::{Entity, HDatabase, Repo, RepoDelegate};
    use futures::executor;
    use knockoff_security::knockoff_security::user_request_account::SessionData;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::any::Any;
    use std::cell::RefCell;
    use std::collections::{HashMap, LinkedList};
    use std::marker::PhantomData;
    use std::ops::Deref;
    use std::pin::Pin;
    use crate::web_framework::context::{ApplicationContext, RequestContext};
    use crate::web_framework::request::request::WebResponse;
    use crate::web_framework::security::authentication::AuthenticationToken;
    use crate::web_framework::security::security_context_holder::SecurityContextHolder;


    #[derive(Serialize, Deserialize, Debug, Copy, Clone, Default)]
    pub struct WebApplication {}

    #[derive(Serialize, Deserialize, Debug, Clone, Default)]
    pub struct HttpSession {
        pub ctx: WebApplication,
        pub session_data: SessionData,
        pub security_context_holder: SecurityContextHolder,
        pub id: Option<String>,
    }

    impl HttpSession {
        pub fn new(
            id: String,
            authentication_token: Option<AuthenticationToken>,
            ctx: WebApplication,
            session_data: SessionData,
        ) -> HttpSession {
            Self {
                ctx,
                session_data,
                security_context_holder: SecurityContextHolder::default(),
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

    impl<'a, R, Request, Response> Action<Request, Response> for SessionFilter<'a, R>
    where
        R: Repo<'a, HttpSession, String>,
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {

        fn do_action(
            &self,
            metadata: EndpointMetadata,
            request: &Option<Request>,
            mut web_request: &WebRequest,
            mut response: &mut WebResponse,
            context: &RequestContext<Request, Response>,
            application_context: &ApplicationContext<Request, Response>
        ) -> Option<Response> {
            if let Some(session) = web_request
                .headers
                .get("R_SESSION_ID")
                .and_then(|session_id| executor::block_on(self.repo.find_by_id(session_id))) {
                response.session = session;
            }
            None
        }

        fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool {
            true
        }

        fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
            true
        }

        fn clone(&self) -> Box<dyn Action<Request, Response>> {
            todo!()
        }
    }
}
