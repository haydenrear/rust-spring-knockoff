pub mod test;
pub mod filter {

    extern crate alloc;
    extern crate core;

    use crate::context::RequestContext;
    use crate::controller::{Dispatcher, PostMethodRequestDispatcher, RequestMethodDispatcher};
    use crate::convert::Registration;
    use crate::http::{HttpMethod};
    use crate::request::request::{EndpointMetadata, HttpRequest, HttpResponse, ResponseWriter};
    use crate::security::security::AuthenticationToken;
    use crate::session::session::HttpSession;
    use alloc::string::String;
    use core::borrow::{Borrow, BorrowMut};
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
        pub fn do_filter(&mut self, request: &HttpRequest, response: &mut HttpResponse) {
            let next = self.next();
            if next != -1 {
                let f = &self.filters[(next - 1) as usize];
                f.filter(request, response, self.clone());
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

    pub trait Action<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default,
    {
        fn do_action(
            &self,
            metadata: EndpointMetadata,
            request: &Option<Request>,
            context: &RequestContext,
        ) -> Option<Response>;
        fn authentication_granted(&self, token: &Option<AuthenticationToken>) -> bool;
    }

    pub struct FilterImpl<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default,
    {
        pub(crate) actions: Box<dyn Action<Request, Response>>,
        pub(crate) dispatcher: Dispatcher,
    }

    pub trait Filter {
        fn filter(&self, request: &HttpRequest, response: &mut HttpResponse, filter: FilterChain);
    }

    impl<Request, Response> Filter for FilterImpl<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default,
    {
        fn filter(
            &self,
            request: &HttpRequest,
            response: &mut HttpResponse,
            mut filter: FilterChain,
        ) {
            self.dispatcher
                .do_request(request.clone(), response, &self.actions);
            filter.do_filter(request, response)
        }
    }
}
