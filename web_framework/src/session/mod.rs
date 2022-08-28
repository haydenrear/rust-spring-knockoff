pub mod repo;
pub mod session {

    extern crate alloc;
    extern crate core;

    use crate::filter::filter::{Filter, FilterChain};
    use crate::request::request::{HttpRequest, HttpResponse};
    use crate::security::security::{AuthenticationToken, AuthenticationTokenImpl};
    use alloc::string::String;
    use core::borrow::Borrow;
    use data_framework::Entity;
    use security_model::SessionData;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::cell::RefCell;
    use std::collections::{HashMap, LinkedList};
    use std::pin::Pin;

    impl Default for WebApplication {
        fn default() -> Self {
            Self {}
        }
    }

    #[derive(Serialize, Deserialize, Debug, Copy, Clone)]
    pub struct WebApplication {}

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct HttpSession {
        ctx: WebApplication,
        session_data: SessionData,
        authentication_token: Option<AuthenticationTokenImpl>,
        pub(crate) id: Option<String>,
    }

    impl HttpSession {
        pub fn new(
            id: String,
            authentication_token: Option<AuthenticationTokenImpl>,
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
                authentication_token: Some(AuthenticationTokenImpl::default()),
                id: Some(String::from("1")),
            }
        }
    }

    pub struct SessionFilter {}

    impl Filter for SessionFilter {
        fn filter(&self, request: HttpRequest, response: HttpResponse, filter: FilterChain) {
            todo!()
        }
    }
}
