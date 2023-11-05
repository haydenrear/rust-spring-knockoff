use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Pointer};
use std::future::Future;
use std::io::Read;
use std::marker::PhantomData;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::ops::Deref;
use std::sync::Arc;
use async_std::io::WriteExt;
use chrono::format::Item;
use circular::Buffer;
use futures::{Sink, SinkExt};
use hyper::{Body, Request, Response, Server};
use async_trait::async_trait;
use hyper::body::HttpBody;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::service::{make_service_fn, Service, service_fn};
use serde::{Deserialize, Serialize};
use serde::de::StdError;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use web_framework::web_framework::context::{Context, UserRequestContext};
use web_framework::web_framework::convert::Registration;
use web_framework::web_framework::http::{
    ProtocolToAdaptFrom, RequestConversionError,
    RequestConverter, RequestStream, ResponseType
};
use web_framework::web_framework::request_context::SessionContext;
use web_framework_shared::{HandlerExecutor, RequestExecutor};
use web_framework_shared::http_method::HttpMethod;
use web_framework_shared::request::{WebRequest, WebResponse};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/hyper_request.log"));

pub struct HyperRequestStream<RequestT, ResponseT, RequestExecutorT, RequestConverterT>
where
    ResponseT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    RequestT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    RequestExecutorT: RequestExecutor<RequestT, ResponseT> + Send + Sync,
    RequestConverterT: RequestConverter<Request<Body>, RequestT, HyperBodyConvertError> + 'static + Send + Sync
{
    pub request_executor: Arc<RequestExecutorT>,
    pub converter: Arc<RequestConverterT>,
    pub response: PhantomData<ResponseT>,
    pub request: PhantomData<RequestT>
}

impl <RequestT, ResponseT, RequestExecutorT, RequestConverterT> HyperRequestStream<RequestT, ResponseT, RequestExecutorT, RequestConverterT>
where
    ResponseT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    RequestT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    RequestExecutorT: RequestExecutor<RequestT, ResponseT> + Send + Sync + 'static,
    RequestConverterT: RequestConverter<Request<Body>, RequestT, HyperBodyConvertError> + 'static + Send + Sync

{
    pub fn new(
        request_executor: RequestExecutorT,
        converter: RequestConverterT
    ) -> Self {
        HyperRequestStream {
            request_executor: request_executor.into(),
            converter: converter.into(),
            response: Default::default(),
            request: Default::default(),
        }
    }
}

pub struct Addr<'a> {
    addr: &'a AddrStream
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct HyperRequestStreamError {
    message: String
}

impl Display for HyperRequestStreamError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <HyperRequestStreamError as Debug>::fmt(self, f)
    }
}



impl HyperRequestStreamError {
    pub fn new(message: &str) -> Self {
        Self {message: message.to_string()}
    }
}

impl StdError for HyperRequestStreamError {
}

impl <RequestT, ResponseT, RequestExecutorT, RequestConverterT>
HyperRequestStream<RequestT, ResponseT, RequestExecutorT, RequestConverterT>
    where
        ResponseT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        RequestT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        RequestExecutorT: RequestExecutor<RequestT, ResponseT> + Send + Sync + 'static,
        RequestConverterT: RequestConverter<Request<Body>, RequestT, HyperBodyConvertError> + 'static + Send + Sync
{

    pub async fn do_run(&mut self) {
        let addr = ([127, 0, 0, 1], 3000).into();

        let service = make_service_fn(|cnn: &AddrStream| {
            let converter = self.converter.clone();
            let request_executor = self.request_executor.clone();
            async move  {
                Ok::<_, Error>(service_fn(move |requested| {
                    let converter = converter.clone();
                    let request_executor = request_executor.clone();
                    async move {
                        converter.from(requested).await
                            .map_err(|e| {
                                error!("Error in service function converting from request: {:?}", e);
                                e
                            })
                            .ok()
                            .map(|mut converted| {
                                let web_response = request_executor.do_request(
                                    converted
                                );
                                serde_json::to_string::<ResponseT>(&web_response)
                                    .map(|res| Response::new(Body::from(res)))
                                    .map_err(|e| {
                                        error!("Error in service function converting response to json: {:?}", e);
                                        HyperRequestStreamError::new("Failed.")
                                    })
                            })
                            .or(Some(Err(HyperRequestStreamError::new("Failed."))))
                            .unwrap()
                    }
                }))
            }
        });

        let server = Server::bind(&addr)
            .serve(service);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}

#[derive(Debug, Default)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       todo!()
    }
}

impl StdError for Error {}

#[derive(Clone)]
pub struct HyperRequestConverter {
}

impl HyperRequestConverter {
    pub fn new() -> Self{
        HyperRequestConverter {}
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HyperBodyConvertError {
    error: &'static str
}

impl HyperBodyConvertError {
    pub fn new(error: &'static str) -> Self{
        Self {
            error
        }
    }
}

impl StdError for HyperBodyConvertError {}

impl RequestConversionError for HyperBodyConvertError {
}

impl Display for HyperBodyConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.error)
    }
}

#[async_trait]
impl <'a> RequestConverter<Request<Body>, WebRequest, HyperBodyConvertError> for HyperRequestConverter
{
    async fn from(&self, in_value: Request<Body>) -> Result<WebRequest,HyperBodyConvertError> {
        let from_headers = in_value.headers().clone();
        let http_body = in_value.into_body();
        hyper::body::to_bytes(http_body).await
            .map(|b| String::from_utf8(b.to_vec()))
            .map(|v| {
                v.map_or_else(|_| WebRequest::default(), |s| {
                    let mut headers = HashMap::new();
                    for header_tuple in from_headers.iter() {
                        header_tuple.1.to_str().ok()
                            .map(|header_value| {
                                headers.insert(header_tuple.0.to_string().clone(), String::from(header_value));
                            });
                    }
                    WebRequest {
                        headers,
                        body: s,
                        metadata: Default::default(),
                        method: HttpMethod::Post,
                        uri: "".to_string(),
                    }
                })
            })
            .or(Err(HyperBodyConvertError::new("Could not convert from Hyper request")))
    }
}

