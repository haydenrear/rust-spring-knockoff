pub mod test;
pub mod filter {

    extern crate alloc;
    extern crate core;

    use crate::web_framework::context::{RequestContextData, UserRequestContext};
    use crate::web_framework::dispatch::FilterExecutor;
    use core::borrow::Borrow;
    use serde::{Deserialize, Serialize};
    use std::cmp::Ordering;
    use std::ops::Deref;
    use std::sync::Arc;
    use web_framework_shared::controller::{HandlerInterceptor, HandlerMethod};
    use web_framework_shared::dispatch_server::Handler;
    use web_framework_shared::request::WebResponse;
    use web_framework_shared::request::WebRequest;

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
        pub(crate) filters: Arc<Vec<Filter<Request, Response>>>,
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

    impl <Request, Response> FilterChain<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
    {
        pub fn do_filter(&self, request: &WebRequest, response: &mut WebResponse, ctx: &RequestContextData<Request, Response>,
                         request_context: &mut Option<Box<UserRequestContext<Request>>>) {
            self.filters.iter()
                .for_each(|f| f.filter(request, response, ctx, request_context));
        }

        pub fn new(mut filters: Vec<Filter<Request, Response>>) -> Self {
            filters.sort_by(|first, second| first.order.cmp(&second.order));
            Self {
                filters: Arc::new(filters)
            }
        }
    }

    #[derive(PartialEq, Clone, Copy)]
    pub enum MediaType {
        Json,
        Xml,
        Html
    }

    // TODO: can make this a macro to remove dyn
    pub struct Filter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    {
        pub(crate) actions: Arc<dyn Handler<Request, Response, UserRequestContext<Request>, RequestContextData<Request, Response>>>,
        pub(crate) dispatcher: Arc<FilterExecutor>,
        pub order: u8
    }

    impl<Request, Response> Eq for Filter<Request, Response>
        where
            Request: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b>,
            Response: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b> {}

    impl<Request, Response> PartialEq<Self> for Filter<Request, Response>
        where
            Request: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b>,
            Response: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b> {
        fn eq(&self, other: &Self) -> bool {
            self.order == other.order
        }
    }

    impl<Request, Response> PartialOrd<Self> for Filter<Request, Response>
        where
            Request: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b>,
            Response: Clone + Default + Send + Serialize + Sync + for<'b> Deserialize<'b>
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.order.partial_cmp(&other.order)
        }
    }

    impl <Request, Response> Ord for Filter<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            self.order.cmp(&other.order)
        }
    }

    impl <Request, Response> Clone for Filter<Request, Response>
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


    impl <Request, Response> Filter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        pub fn new(actions: Arc<dyn Handler<Request, Response, UserRequestContext<Request>, RequestContextData<Request, Response>>>,
                   order: Option<u8>,
                   dispatcher: Arc<FilterExecutor>) -> Self {
            Self {
                actions,
                dispatcher,
                order: order.or(Some(0)).unwrap()
            }
        }
    }

    impl<Request, Response> Filter<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    {
        fn filter(
            &self,
            request: &WebRequest,
            response: &mut WebResponse,
            ctx: &RequestContextData<Request, Response>,
            request_context: &mut Option<Box<UserRequestContext<Request>>>
        ) {
            self.dispatcher
                .do_request(request, response, self.actions.clone(), ctx, request_context);
        }
    }


    impl <Request, Response> HandlerInterceptor<UserRequestContext<Request>, RequestContextData<Request, Response>> for Filter<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    {
        fn pre_handle(&self,
                      request: &WebRequest,
                      response: &mut WebResponse,
                      data: &mut HandlerMethod<UserRequestContext<Request>>,
                      ctx: &RequestContextData<Request, Response>
        ) {
            self.actions.do_action(
                request,
                response,
                ctx,
                &mut data.request_ctx_data);
        }

        fn post_handle(&self,
                       request: &WebRequest,
                       response: &mut WebResponse,
                       data: &mut HandlerMethod<UserRequestContext<Request>>,
                       ctx: &RequestContextData<Request, Response>
        ) {
            todo!()
        }

        fn after_completion(&self,
                            request: &WebRequest,
                            response: &mut WebResponse,
                            data: &mut HandlerMethod<UserRequestContext<Request>>,
                            ctx: &RequestContextData<Request, Response>
        ) {
            todo!()
        }
    }

}
