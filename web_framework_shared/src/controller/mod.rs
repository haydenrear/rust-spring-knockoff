use std::marker::PhantomData;
use std::sync::Arc;
use std::task::Context;
use serde::{Deserialize, Serialize};
use crate::matcher::{AntPathRequestMatcher, Matcher};
use crate::request::{EndpointMetadata, WebRequest, WebResponse};

/// will be impl Action<> for HandlerMethod and add metadata about it
pub struct HandlerMethod<RequestCtxData: Data + ?Sized>
{
    pub endpoint_metadata: EndpointMetadata,
    pub request_ctx_data: Option<Box<RequestCtxData>>
}

impl <RequestCtxData: Data + ?Sized + Default> Default for HandlerMethod<RequestCtxData> {
    fn default() -> Self {
        Self {
            endpoint_metadata: EndpointMetadata::default(),
            request_ctx_data: None
        }
    }
}

pub trait Data: Send + Sync {
}

pub trait ContextData: Send + Sync {
}


#[derive(Default, Clone)]
pub struct HandlerExecutorStruct<H: HandlerExecutor<T, Ctx> + ?Sized, T: Data + ?Sized, Ctx: ContextData + ?Sized> {
    pub handler_executor: Arc<H>,
    pub phantom_data_t: PhantomData<T>,
    pub phantom_data_ctx: PhantomData<Ctx>
}

pub struct HandlerExecutionChain<T: Data + ?Sized, Ctx: ContextData + ?Sized>
{
    pub interceptors: Arc<Vec<Box<dyn HandlerInterceptor<T, Ctx>>>>,
    pub request_matchers: Vec<AntPathRequestMatcher>,
    pub context: Arc<Ctx>,
    pub handler_executor: Arc<HandlerExecutorStruct<dyn HandlerExecutor<T, Ctx>, T, Ctx>>
}


impl <D, C> HandlerExecutionChain<D, C>
    where D: Data + Send + Sync + ?Sized + Default, C: ContextData + Send + Sync + ?Sized
{
    pub fn create_handler_method(&self) -> HandlerMethod<D> {
        HandlerMethod::default()
    }

}

impl <T: Data + ?Sized, Ctx: ContextData + ?Sized> HandlerExecutionChain<T, Ctx> {

    pub fn matches(&self, request: &WebRequest) -> bool {
        self.request_matchers.iter().any(|r| r.matches(request))
    }
}

pub trait RequestExecutor<WebRequest, WebResponse>: Send + Sync
    where
        WebRequest: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        WebResponse: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    fn do_request(&self, response_writer_type: WebRequest) -> WebResponse;
}

impl <D, C> RequestExecutor<WebRequest, WebResponse> for HandlerExecutionChain<D, C>
    where D: Data + Send + Sync + ?Sized + Default, C: ContextData + Send + Sync + ?Sized
{
    fn do_request(&self, mut web_request: WebRequest) -> WebResponse {
        let mut response = WebResponse::default();
        let mut handler = HandlerMethod::default();

        self.do_request_inner(&mut web_request, &mut response, &mut handler);

        response
    }
}

impl<D, C> HandlerExecutionChain<D, C> where C: ?Sized + ContextData + Send + Sync, D: ?Sized + Data + Send + Sync {
    fn do_request_inner(&self, web_request: &mut WebRequest, mut response: &mut WebResponse, mut handler: &mut HandlerMethod<D>) {
        self.interceptors
            .iter()
            .for_each(|i|
                i.pre_handle(&web_request, &mut response, &mut handler, &self.context)
            );

        self.handler_executor.handler_executor
            .execute_handler(&handler, &self.context, &mut response, &web_request);

        self.interceptors
            .iter()
            .for_each(|i|
                i.post_handle(&web_request, &mut response, &mut handler, &self.context)
            );
    }
}

pub trait HandlerExecutor<D: Data + Send + Sync + ?Sized, Ctx: ContextData + Send + Sync + ?Sized>: Send + Sync {
    fn execute_handler(&self, handler: &HandlerMethod<D>, ctx: &Ctx, response: &mut WebResponse, request: &WebRequest);
}

pub trait HandlerMethodFactory<RequestCtxData: Data + ?Sized> {
    fn get_handler_method(&self) -> HandlerMethod<RequestCtxData>;
}

impl <T: Data + ?Sized, RequestCtxData: ContextData + ?Sized> HandlerExecutionChain<T, RequestCtxData> {

    pub fn pre_handle(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &RequestCtxData) {
        self.interceptors.iter()
            .for_each(|i| i.pre_handle(request, response, data, ctx));
    }

    fn post_handle(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &RequestCtxData) {
        self.interceptors.iter()
            .for_each(|i| i.post_handle(request, response, data, ctx));
    }

    fn after_completion(&self, request: &WebRequest, response: &mut WebResponse, data: &mut HandlerMethod<T>, ctx: &RequestCtxData) {
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