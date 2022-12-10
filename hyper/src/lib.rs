use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::io::{Read};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::ops::Deref;
use std::task::{Context, Poll};
use async_std::io::WriteExt;
use async_std::prelude::StreamExt;
use chrono::format::Item;
use circular::Buffer;
use futures::{Sink, SinkExt};
use hyper::{Body, Request, Response, Server};
use async_trait::async_trait;
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::service::{make_service_fn, Service, service_fn};
use serde::{Deserialize, Serialize};
use serde::de::StdError;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use web_framework::context::ApplicationContext;
use web_framework::http::{ProtocolToAdaptFrom, RequestConverter, RequestExecutor, RequestExecutorImpl, RequestStream, ResponseType, WriteToConnection};
use web_framework::request::request::{WebRequest, WebResponse};
use web_framework::security::security::Converter;

pub struct HyperHandlerAdapter<'a>
{
    request_stream: &'a dyn RequestStream<'a, WebRequest, &'a [u8]>
}

pub struct HyperRequestStream {
    request_executor: RequestExecutorImpl,
    converter: HyperRequestConverter,
}

impl HyperRequestStream {
    fn new() -> Self {
        HyperRequestStream {
            request_executor: RequestExecutorImpl {
                ctx: ApplicationContext::new()
            },
            converter: HyperRequestConverter::new()
        }
    }
}


impl <'a> HyperRequestStream {
    async fn do_run(&'static self) {
        let addr = ([127, 0, 0, 1], 3000).into();
        let service = make_service_fn(|cnn: &AddrStream| async move {
            Ok::<_, Error>(service_fn(move |rqst| async move {
                let request = self.converter.from(rqst);
                let web_response = self.request_executor.do_request(request);
                Ok::<_, Error>(Response::new(Body::from(web_response.response)))
            }))
        });

        let server = Server::bind(&addr).serve(service);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }
}

#[test]
fn test_hyper_request_stream() {

}

pub struct Error;

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl StdError for Error {

}


#[async_trait]
impl <'a> RequestStream<'a, ResponseType<'a>, &'a [u8]> for HyperRequestStream
{
    async fn next(&self) -> ResponseType<'a> {
        // loop {
        //     if !self.buffer.empty() {
                // let mut str = "".to_string();
                // self.buffer.read_to_string(&mut str);
                // self.converter.from(serialized_request)
            // }
        // }
        todo!()
    }
}

#[async_trait]
impl <'a> RequestStream<'a, ResponseType<'a>, &'a [u8]>
for HyperHandlerAdapter<'a>
{
    async fn next(&self) -> ResponseType<'a> {
        todo!()
    }
}

#[derive(Clone)]
pub struct HyperRequestConverter {
}

impl HyperRequestConverter {
    fn new() -> Self{
        HyperRequestConverter {}
    }
}

impl <'a> RequestConverter<Request<Body>, WebRequest> for HyperRequestConverter
{
    fn from(&self, in_value: Request<Body>) -> WebRequest {
        todo!()
    }
}

