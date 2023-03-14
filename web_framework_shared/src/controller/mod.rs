use std::sync::Arc;
use crate::request::WebRequest;

pub trait HandlerMapping {
    fn get_handler(&self, request: WebRequest) -> HandlerExecutionChain<dyn HandlerInterceptor>;
}

pub struct HandlerExecutionChain<T: HandlerInterceptor + ?Sized> {
    interceptors: Arc<T>
}

impl <T: HandlerInterceptor + ?Sized> HandlerExecutionChain<T> {
    fn matches(&self, request: WebRequest) -> bool {
        self.interceptors.matches(&request)
    }
}

pub trait HandlerInterceptor {
    fn handle(&self, request: WebRequest);
    fn matches(&self, request: &WebRequest) -> bool;
}

#[test]
fn test_handler_mapping() {

    pub struct TestHandlerInterceptor;

    impl HandlerInterceptor for TestHandlerInterceptor {
        fn handle(&self, request: WebRequest) {
            todo!()
        }

        fn matches(&self, request: &WebRequest) -> bool {
            todo!()
        }
    }

    pub struct TestHandlerMapping;

    impl HandlerMapping for TestHandlerMapping {

        fn get_handler(&self, request: WebRequest) -> HandlerExecutionChain<dyn HandlerInterceptor> {
            todo!()
        }
    }

    let interceptor = HandlerExecutionChain { interceptors: Arc::new(TestHandlerInterceptor{}) };

}