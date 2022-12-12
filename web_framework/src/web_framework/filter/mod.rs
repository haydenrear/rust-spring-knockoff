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
    use std::any::Any;
    use std::cell::RefCell;
    use std::cmp::Ordering;
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, LinkedList};
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut, Index};
    use std::path::Iter;
    use std::sync::{Arc, Mutex};
    use crate::web_framework::filter;

    impl <Request, Response> Default for FilterChain<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
    {
        fn default() -> Self {
            Self {
                filters: Arc::new(vec![])
            }
        }
    }

    pub struct FilterChain< Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
    {
        pub(crate) filters: Arc<Vec<RequestResponseActionFilter<Request, Response>>>,
    }

    impl <Request, Response> Clone for FilterChain<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
    {
        fn clone(&self) -> Self {
            let mut to_sort = self.filters.deref().clone();
            to_sort.sort();
            Self {
                filters: Arc::new(to_sort)
            }
        }
    }

    // TODO: make the self reference non-mutable - otherwise it can only be run one at a time,
    // resulting in new filter
    impl <Request, Response> FilterChain<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
    {
        pub fn do_filter(&self, request: &WebRequest, response: &mut WebResponse, ctx: &ApplicationContext<Request, Response>) {
            self.filters.iter()
                .for_each(|f| f.filter(request, response, ctx));
        }

        pub fn new(filters: Vec<RequestResponseActionFilter<Request, Response>>) -> Self {
            Self {
                filters: Arc::new(filters)
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
            web_request: &WebRequest,
            response: &mut WebResponse,
            context: &RequestContext,
            application_context: &ApplicationContext<Request, Response>
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
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        pub(crate) actions: Box<dyn Action<Request, Response>>,
        pub(crate) dispatcher: Dispatcher,
        pub order: u8
    }

    impl<Request, Response> Eq for RequestResponseActionFilter<Request, Response> where Request: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b>, Response: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b> {}

    impl<Request, Response> PartialEq<Self> for RequestResponseActionFilter<Request, Response> where Request: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b>, Response: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b> {
        fn eq(&self, other: &Self) -> bool {
            self.order == other.order
        }
    }

    impl<Request, Response> PartialOrd<Self> for RequestResponseActionFilter<Request, Response> where Request: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b>, Response: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.order.partial_cmp(&other.order)
        }
    }

    impl <Request, Response> Ord for RequestResponseActionFilter<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            self.order.cmp(&other.order)
        }
    }

    impl <Request, Response> Clone for RequestResponseActionFilter<Request, Response>
        where
              Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
              Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,

    {
        fn clone(&self) -> Self {
            Self {
                actions: self.actions.clone(),
                dispatcher: self.dispatcher.clone(),
                order: self.order
            }
        }
    }

    impl <Request, Response> RequestResponseActionFilter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        pub fn new(action: Box<dyn Action<Request, Response>>, order: Option<u8>) -> Self {
            Self {
                actions: action,
                dispatcher: Dispatcher::default(),
                order: order.or(Some(0)).unwrap()
            }
        }
    }

    impl<Request, Response> RequestResponseActionFilter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    {
        fn filter(
            &self,
            request: &WebRequest,
            response: &mut WebResponse,
            ctx: &ApplicationContext<Request, Response>
        ) {
            self.dispatcher
                .do_request(request.clone(), response, &self.actions, ctx);
        }
    }
}
