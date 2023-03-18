use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Pointer};
use std::future::Future;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::ops::Deref;
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
    RequestConverter, RequestExecutor, RequestExecutorImpl,
    RequestStream, ResponseType
};
use web_framework::web_framework::request_context::SessionContext;
use web_framework_shared::http_method::HttpMethod;
use web_framework_shared::request::{WebRequest, WebResponse};

pub struct HyperRequestStream<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static

{
    pub request_executor: RequestExecutorImpl<Request, Response>,
    pub converter: HyperRequestConverter,
}

impl <Request, Response> HyperRequestStream<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static

{
    pub fn new(request_executor: RequestExecutorImpl<Request, Response>) -> Self {
        HyperRequestStream {
            request_executor,
            converter: HyperRequestConverter::new()
        }
    }
}

pub struct Addr<'a> {
    addr: &'a AddrStream
}

impl <HRequest, HResponse> HyperRequestStream<HRequest, HResponse>
    where
        HResponse: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        HRequest: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{

    pub async fn do_run(&mut self) {
        let addr = ([127, 0, 0, 1], 3000).into();

        let service = make_service_fn(|cnn: &AddrStream| {
            let converter = self.converter.clone();
            let request_executor = self.request_executor.clone();
            async move  {
                Ok::<_, Error>(service_fn(move |rqst| {
                    let converter_cloned = converter.clone();
                    let request_exec_cloned = request_executor.clone();
                    async move {
                        converter_cloned.from(rqst).await
                            .map(|converted| {
                                // TODO: use generated dispatcher
                                // let web_response = request_exec_cloned.do_request(
                                //     converted,
                                //     &mut UserRequestContext { request_context: RequestContext { http_session: Default::default() }, request: None }
                                // );
                                // Response::new(Body::from(web_response.response))
                                Response::new(Body::from("hello".to_string()))
                            })
                            .or(Err(HyperBodyConvertError { error: "failure" }))
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
            .map(|b| {
                String::from_utf8(b.to_vec())
            })
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
                        headers: headers,
                        body: s,
                        metadata: Default::default(),
                        method: HttpMethod::Post,
                        uri: "".to_string(),
                    }
                })
            })
            .or(Err(HyperBodyConvertError{error: "Could not convert from Hyper request"}))
    }
}

