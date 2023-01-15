pub mod repo;
pub mod session {

    extern crate alloc;
    extern crate core;

    use crate::web_framework::filter::filter::{Action, FilterChain};
    use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse};
    use crate::web_framework::security::security::{Authentication, AuthenticationToken, AuthenticationType};
    use alloc::string::String;
    use async_std::task as async_task;
    use core::borrow::Borrow;
    use data_framework::{Entity, HDatabase, Repo, RepoDelegate};
    use futures::executor;
    use security_model::SessionData;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::any::Any;
    use std::cell::RefCell;
    use std::collections::{HashMap, LinkedList};
    use std::marker::PhantomData;
    use std::ops::Deref;
    use std::pin::Pin;
    use crate::web_framework::context::{ApplicationContext, RequestContext};

    impl Default for WebApplication {
        fn default() -> Self {
            Self {}
        }
    }

    #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
    pub struct WebApplication {}

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct HttpSession {
        pub ctx: WebApplication,
        pub session_data: SessionData,
        pub authentication_token: Option<AuthenticationToken<AuthenticationType>>,
        pub id: Option<String>,
    }

    impl HttpSession {
        pub fn new(
            id: String,
            authentication_token: Option<AuthenticationToken<AuthenticationType>>,
            ctx: WebApplication,
            session_data: SessionData,
        ) -> HttpSession {
            Self {
                ctx: ctx,
                session_data: session_data,
                authentication_token: authentication_token,
                id: Some(id),
            }
        }
    }

    impl Default for HttpSession {
        fn default() -> Self {
            Self {
                ctx: WebApplication::default(),
                session_data: SessionData::default(),
                authentication_token: Some(AuthenticationToken::default()),
                id: Some(String::from("1")),
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
                .and_then(|session_id| executor::block_on(self.repo.find_by_id(session_id.clone()))) {
                response.session = session;
            }
            None
        }

        fn authentication_granted(&self, token: &Option<AuthenticationToken<AuthenticationType>>) -> bool {
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
