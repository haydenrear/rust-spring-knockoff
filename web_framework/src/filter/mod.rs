pub mod test;
pub mod filter {

    extern crate alloc;
    extern crate core;

    use crate::context::{ApplicationContext, RequestContext};
    use crate::dispatch::{Dispatcher, PostMethodRequestDispatcher, RequestMethodDispatcher};
    use crate::convert::Registration;
    use crate::http::{Connection, HttpMethod};
    use crate::request::request::{EndpointMetadata, WebRequest, WebResponse, ResponseWriter};
    use crate::security::security::AuthenticationToken;
    use crate::session::session::HttpSession;
    use alloc::string::String;
    use core::borrow::{Borrow, BorrowMut};
    use std::cmp::Ordering;
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, LinkedList};
    use std::ops::{Deref, Index};
    use std::path::Iter;

    pub struct FilterChain {
        filters: Vec<Box<dyn Filter>>
    }

    impl Clone for FilterChain {
        fn clone(&self) -> Self {
            let filters = self.filters.iter()
                .map(|f| f.replicate())
                .collect::<Vec<Box<dyn Filter>>>();
            Self {
                filters: filters
            }
        }
    }

    // TODO: make the self reference non-mutable - otherwise it can only be run one at a time,
    // resulting in new filter
    impl FilterChain {
        pub fn do_filter(&mut self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext) {
            for f in &self.filters {
                f.filter(request, response, ctx);
            }
        }

        pub fn new(filters: Vec<Box<dyn Filter>>) -> Self {
            Self {
                filters: filters
            }
        }
    }

    #[derive(PartialEq)]
    pub enum MediaType {
        Json,
        Xml,
    }

    pub trait DispatcherContainer {
        fn dispatcher<Response, Request>(
            &self,
            method: HttpMethod,
        ) -> &'static dyn Action<Request, Response>;
    }

    pub trait Action<Request, Response>: Send + Sync
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        fn do_action(
            &self,
            metadata: EndpointMetadata,
            request: &Option<Request>,
            context: &RequestContext,
        ) -> Option<Response>;

        fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool;

        /**
        determines if it matches endpoint, http method, etc.
        */
        fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool;

        fn replicate(&self) -> Box<dyn Action<Request, Response>>;

    }


    /***
    Every "controller endpoint" will create one of these.
     */
    pub struct RequestResponseActionFilter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        pub(crate) actions: Box<dyn Action<Request, Response>>,
        pub(crate) dispatcher: Dispatcher,
    }

    impl <Request, Response> RequestResponseActionFilter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        pub fn new(action: Box<dyn Action<Request, Response>>) -> Self {
            Self {
                actions: action,
                dispatcher: Dispatcher::default()
            }
        }
    }

    pub trait Filter : Send + Sync{
        fn filter(&self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext);
        fn replicate(&self) -> Box<dyn Filter>;
    }

    impl<Request, Response> Filter for RequestResponseActionFilter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    {
        fn filter(
            &self,
            request: &WebRequest,
            response: &mut WebResponse,
            ctx: &ApplicationContext
        ) {
            self.dispatcher
                .do_request(request.clone(), response, &self.actions);
        }

        fn replicate(&self) -> Box<dyn Filter> {
            Box::new(RequestResponseActionFilter {
                actions: self.actions.replicate(),
                dispatcher: Default::default(),
            })
        }
    }
}
