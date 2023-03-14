use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::matcher::AntPathRequestMatcher;
use crate::request::{EndpointMetadata, WebRequest, WebResponse};

/// will be impl Action<> for HandlerMethod and add metadata about it
pub struct HandlerMethod<RequestCtxData: Data>
{
    pub endpoint_metadata: EndpointMetadata,
    pub request_ctx_data: RequestCtxData
}

pub trait Data {
}

pub trait ContextData {
}

pub struct HandlerExecutionChain<T: Data, Ctx: ContextData>

{
    pub interceptors: Arc<Vec<Box<dyn HandlerInterceptor<T, Ctx>>>>,
    pub handler: HandlerMethod<T>,
    pub request_matchers: Vec<AntPathRequestMatcher>
}

impl <T: Data, RequestCtxData: ContextData> HandlerExecutionChain<T, RequestCtxData> {

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

pub trait HandlerInterceptor<T: Data, Ctx: ContextData> {
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