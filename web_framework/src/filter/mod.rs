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

    #[derive(Clone)]
    pub struct FilterChain<'a> {
        filters: Vec<&'a dyn Filter>,
        pub(crate) num: usize,
    }

    // TODO: make the self reference non-mutable - otherwise it can only be run one at a time,
    // resulting in new filter
    impl<'a> FilterChain<'a> {
        pub fn do_filter(&mut self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext) {
            let next = self.next();
            if next != -1 {
                let f = &self.filters[(next - 1) as usize];
                f.filter(request, response, self.clone(), ctx);
                if self.num >= self.filters.len() {
                    self.num = 0;
                }
            }
        }

        pub(crate) fn next(&mut self) -> i64 {
            if self.filters.len() > self.num {
                self.num += 1;
                return self.num as i64;
            } else {
                -1
            }
        }

        pub fn new(filters: Vec<&'a dyn Filter>) -> Self {
            Self {
                filters: filters,
                num: 0,
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
        fn filter(&self, request: &WebRequest, response: &mut WebResponse, filter: FilterChain, ctx: &ApplicationContext);
    }

    impl<Request, Response> Filter for RequestResponseActionFilter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        fn filter(
            &self,
            request: &WebRequest,
            response: &mut WebResponse,
            mut filter: FilterChain,
            ctx: &ApplicationContext
        ) {
            self.dispatcher
                .do_request(request.clone(), response, &self.actions);
            filter.do_filter(request, response, ctx)
        }
    }
}
