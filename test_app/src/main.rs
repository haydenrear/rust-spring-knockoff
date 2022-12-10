use lazy_static::lazy_static;
use hyper::{HyperRequestConverter, HyperRequestStream};
use web_framework::context::ApplicationContext;
use web_framework::http::RequestExecutorImpl;
use std::sync::Arc;

lazy_static!(pub static ref RUNNER: Arc<HyperRequestStream> =
    Arc::new(HyperRequestStream {
        request_executor: RequestExecutorImpl {
            ctx: ApplicationContext::new()
        },
        converter: HyperRequestConverter::new()
    });
);

#[tokio::main]
async fn main() {
    RUNNER.do_run().await;
}