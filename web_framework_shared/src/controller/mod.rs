use std::marker::PhantomData;
use std::sync::Arc;
use std::task::Context;
use http::Request;
use serde::{Deserialize, Serialize};
use crate::Handler;
use crate::matcher::{AntPathRequestMatcher, Matcher};
use crate::request::{EndpointMetadata, WebRequest, WebResponse};

/// will be impl Action<> for HandlerMethod and add metadata about it
pub struct HandlerMethod<RequestCtxData: Data + ?Sized>
{
    pub request_ctx_data: Option<Box<RequestCtxData>>
}

impl<RequestCtxData: Data> HandlerMethod<RequestCtxData> {
    pub fn new(request_ctx_data: Box<RequestCtxData>) -> Self {
        Self {
            request_ctx_data: Some(request_ctx_data),
        }
    }
}

impl <RequestCtxData: Data + ?Sized + Default> Default for HandlerMethod<RequestCtxData> {
    fn default() -> Self {
        Self {
            request_ctx_data: None
        }
    }
}

pub trait Data: Send + Sync {
}

pub trait ContextData: Data {
}


#[derive(Default, Clone)]
pub struct HandlerExecutorStruct<H: HandlerExecutor<T, Ctx, Request, Response> + ?Sized, T: Data + ?Sized, Ctx: ContextData + ?Sized, Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub handler_executor: Arc<H>,
    pub phantom_data_t: PhantomData<T>,
    pub phantom_data_ctx: PhantomData<Ctx>,
    pub response: PhantomData<Response>,
    pub request: PhantomData<Request>,
}

pub struct HandlerExecutionChain<T: Data + ?Sized, Ctx: ContextData + ?Sized, Request, Response, H: HandlerExecutor<T, Ctx, Request, Response>>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub interceptors: Arc<Vec<Box<dyn HandlerInterceptor<T, Ctx>>>>,
    pub request_matchers: Vec<AntPathRequestMatcher>,
    pub context: Arc<Ctx>,
    pub handler_executor: Arc<HandlerExecutorStruct<H, T, Ctx, Request, Response>>
}

impl<T: Data + ?Sized, Ctx: ContextData + ?Sized, Request, Response, H: HandlerExecutor<T, Ctx, Request, Response>> Handler<Request, Response, T, Ctx>
for HandlerExecutionChain<T, Ctx, Request, Response, H>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    fn do_action(&self, web_request: &WebRequest, response: &mut WebResponse,
                 context: &Ctx, request_context: &mut Option<Box<T>>) -> Option<Response> {
        self.handler_executor.handler_executor.do_action(web_request, response, context, request_context)
    }

    fn authentication_granted(&self, token: &Option<Box<T>>) -> bool {
        self.handler_executor.handler_executor.authentication_granted(token)
    }

    fn matches(&self, endpoint_metadata: &EndpointMetadata) -> bool {
        self.handler_executor.handler_executor.matches(endpoint_metadata)
    }
}

impl <T, Ctx, Request, Response, H: HandlerExecutor<T, Ctx, Request, Response>> HandlerExecutionChain<T, Ctx, Request, Response, H>
    where
        T: Data + Send + Sync + ?Sized + Default, Ctx: ContextData + Send + Sync + ?Sized,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub fn create_handler_method(&self) -> HandlerMethod<T> {
        HandlerMethod::default()
    }
}

impl <T: Data + ?Sized, Ctx: ContextData + ?Sized, Request, Response, H: HandlerExecutor<T, Ctx, Request, Response>>
HandlerExecutionChain<T, Ctx, Request, Response, H>
where
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{

    pub fn matches(&self, request: &WebRequest) -> bool {
        self.request_matchers.iter().any(|r| r.matches(request))
    }
}

impl<T, Ctx, Request, Response, H: HandlerExecutor<T, Ctx, Request, Response>> HandlerExecutionChain<T, Ctx, Request, Response, H>
where
    Ctx: ?Sized + ContextData + Send + Sync, T: ?Sized + Data + Send + Sync ,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync

{
    fn do_request_inner(&self, web_request: &mut WebRequest, mut response: &mut WebResponse, mut handler: &mut HandlerMethod<T>) {
        self.interceptors
            .iter()
            .for_each(|i|
                i.pre_handle(&web_request, &mut response, &mut handler, &self.context));

        self.handler_executor.handler_executor
            .execute_handler(&handler, &mut response, &web_request);

        self.interceptors
            .iter()
            .for_each(|i|
                i.post_handle(&web_request, &mut response, &mut handler, &self.context));
    }
}

pub trait HandlerExecutor<D: Data + Send + Sync + ?Sized, Ctx: ContextData + Send + Sync + ?Sized, Request, Response>: Send + Sync + Handler<Request, Response, D, Ctx>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
{
    fn execute_handler(&self, handler: &HandlerMethod<D>, response: &mut WebResponse, request: &WebRequest) -> Option<Response>;
}

pub trait HandlerMethodFactory<RequestCtxData: Data + ?Sized> {
    fn get_handler_method(&self) -> HandlerMethod<RequestCtxData>;
}

impl <T: Data + ?Sized, Ctx: ContextData + ?Sized, Request, Response, H: HandlerExecutor<T, Ctx, Request, Response>>
HandlerExecutionChain<T, Ctx, Request, Response, H>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
{

    pub fn pre_handle(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &Ctx) {
        self.interceptors.iter()
            .for_each(|i| i.pre_handle(request, response, data, ctx));
    }

    fn post_handle(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &Ctx) {
        self.interceptors.iter()
            .for_each(|i| i.post_handle(request, response, data, ctx));
    }

    fn after_completion(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &Ctx) {
        self.interceptors.iter()
            .for_each(|i| i.after_completion(request, response, data, ctx));
    }
}

pub trait HandlerAdapter<T, Ctx, RequestCtx> {
    fn handle(&self, request: &WebRequest, response: &mut WebResponse,
              ctx: Ctx, request_ctx: &mut RequestCtx, handler: T);
}

pub trait HandlerInterceptor<T: Data + ?Sized, Ctx: ContextData + ?Sized>: Send + Sync {
    fn pre_handle(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &Ctx);
    fn post_handle(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &Ctx);
    fn after_completion(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &Ctx);
}

#[test]
fn test_handler_mapping() {

    pub struct TestHandlerInterceptor;

    // impl HandlerInterceptor for TestHandlerInterceptor {
    //     fn matches(&self, request: &WebRequest) -> bool {
    //         todo!()
    //     }
    // }
    //
    // pub struct TestHandlerMapping;
    //
    // impl HandlerMapping for TestHandlerMapping {
    //
    //     fn get_handler(&self, request: WebRequest) -> HandlerExecutionChain<dyn HandlerInterceptor> {
    //         todo!()
    //     }
    // }
    //
    // let interceptor = HandlerExecutionChain { interceptors: Arc::new(TestHandlerInterceptor{}) };
    //
}