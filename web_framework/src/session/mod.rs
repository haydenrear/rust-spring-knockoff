pub mod repo;
pub mod session {

    extern crate alloc;
    extern crate core;

    use crate::filter::filter::{Filter, FilterChain};
    use crate::request::request::{WebRequest, WebResponse};
    use crate::security::security::{Authentication, AuthenticationToken};
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
    use crate::context::ApplicationContext;

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
        pub authentication_token: Option<AuthenticationToken>,
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

    impl<'a, R> Filter for SessionFilter<'a, R>
    where
        R: Repo<'a, HttpSession, String>,
    {
        fn filter(
            &self,
            request: &WebRequest,
            response: &mut WebResponse,
            mut filter: FilterChain,
            ctx: &ApplicationContext
        ) {
            if let Some(session) = request
                .headers
                .get("R_SESSION_ID")
                .and_then(|session_id| executor::block_on(self.repo.find_by_id(session_id.clone()))) {
                response.session = session;
            }
            filter.do_filter(request, response, ctx);
        }
    }
}
