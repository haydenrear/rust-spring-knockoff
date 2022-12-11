use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Pointer};
use std::future::Future;
use std::io::{Read};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::ops::Deref;
use std::task::{Context, Poll};
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
use web_framework::context::ApplicationContext;
use web_framework::convert::Registration;
use web_framework::filter::filter::Filter;
use web_framework::http::{
    HttpMethod, ProtocolToAdaptFrom, RequestConversionError,
    RequestConverter, RequestExecutor, RequestExecutorImpl,
    RequestStream, ResponseType
};
use web_framework::request::request::{WebRequest, WebResponse};
use web_framework::security::security::Converter;

pub struct HyperHandlerAdapter<'a>
{
    request_stream: &'a dyn RequestStream<'a, WebRequest, &'a [u8]>
}

pub struct HyperRequestStream {
    pub request_executor: RequestExecutorImpl,
    pub converter: HyperRequestConverter,
}

impl HyperRequestStream {
    pub fn new() -> Self {
        HyperRequestStream {
            request_executor: RequestExecutorImpl {
                ctx: ApplicationContext::new()
            },
            converter: HyperRequestConverter::new()
        }
    }
}

impl <'a> Registration<'a, dyn Filter> for HyperRequestStream
where 'a: 'static
{
    fn register(&mut self, converter: &'a dyn Filter) {
        self.request_executor.ctx.register(converter);
    }
}

impl HyperRequestStream {
    pub async fn do_run(&'static self) {
        let addr = ([127, 0, 0, 1], 3000).into();

        let service = make_service_fn(|cnn: &AddrStream| async move {
            Ok::<_, Error>(service_fn(move |rqst| async move {
                self.converter.from(rqst).await
                    .map(|converted | {
                        let web_response = self.request_executor.do_request(converted);
                        Response::new(Body::from(web_response.response))
                    })
                    .or(Err(HyperBodyConvertError{error: "failure"}))
            }))
        });

        let server = Server::bind(&addr).serve(service);

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
        let http_body = in_value.into_body();
        hyper::body::to_bytes(http_body).await
            .map(|b| {
                String::from_utf8(b.to_vec())
            })
            .map(|v| {
                v.map_or_else(|_| WebRequest::default(), |s| {
                    WebRequest {
                        headers: Default::default(),
                        body: s,
                        metadata: Default::default(),
                        method: HttpMethod::Post,
                    }
                })
            })
            .or(Err(HyperBodyConvertError{error: "Could not convert from Hyper request"}))
    }
}

