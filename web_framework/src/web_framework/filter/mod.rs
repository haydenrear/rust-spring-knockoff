pub mod test;
pub mod filter {

    extern crate alloc;
    extern crate core;

    use crate::web_framework::context::{ApplicationContext, RequestContext};
    use crate::web_framework::dispatch::{Dispatcher, PostMethodRequestDispatcher, RequestMethodDispatcher};
    use crate::web_framework::convert::Registration;
    use crate::web_framework::http::{Connection, HttpMethod};
    use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse, ResponseWriter};
    use crate::web_framework::security::security::AuthenticationToken;
    use crate::web_framework::session::session::HttpSession;
    use alloc::string::String;
    use core::borrow::{Borrow, BorrowMut};
    use std::cmp::Ordering;
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, LinkedList};
    use std::ops::{Deref, Index};
    use std::path::Iter;
    use std::sync::Arc;

    pub struct FilterChain<'a> {
        filters: Vec<&'a Option<Box<dyn Filter>>>
    }

    impl <'a> Clone for FilterChain<'a> {
        fn clone(&self) -> Self {
            let filters = self.filters.clone();
            Self {
                filters: filters
            }
        }
    }

    // TODO: make the self reference non-mutable - otherwise it can only be run one at a time,
    // resulting in new filter
    impl <'a> FilterChain<'a> {
        pub fn do_filter(&mut self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext) {
            for f in &self.filters {
                match f {
                    None => {}
                    Some(found) => {
                        found.filter(request, response, ctx)
                    }
                }
            }
        }

        pub fn new(filters: Vec<&'a Option<Box<dyn Filter>>>) -> Self {
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

        fn clone(&self) -> Box<dyn Action<Request, Response>>;

    }

    /***
    Every "controller endpoint" will create one of these.
     */
    pub struct RequestResponseActionFilter<Request, Response>
    where Self: 'static,
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        pub(crate) actions: Box<dyn Action<Request, Response>>,
        pub(crate) dispatcher: Dispatcher,
    }

    impl <Request, Response> Clone for RequestResponseActionFilter<Request, Response>
        where Self: 'static,
              Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
              Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
    {
        fn clone(&self) -> Self {
            Self {
                actions: self.actions.clone(),
                dispatcher: self.dispatcher.clone()
            }
        }
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
    }
}
