use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Pointer};
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use futures::{Sink};
use hyper::{Body, Method, Request, Response, Server};
use async_trait::async_trait;
use hyper::body::HttpBody;
use hyper::server::conn::{AddrStream};
use hyper::service::{make_service_fn, Service, service_fn};
use serde::{Deserialize, Serialize};
use serde::de::StdError;
use web_framework::web_framework::http::{
    RequestConversionError,
    RequestConverter,
};
use web_framework_shared::{ContextData, Data, EndpointMetadata, HandlerExecutor};
use web_framework_shared::http_method::HttpMethod;
use web_framework_shared::request::{WebRequest};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use web_framework::web_framework::convert::{RequestTypeExtractor};
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/hyper_request.log"));

pub struct HyperRequestStream<RequestT, ResponseT, RequestExecutorT, RequestConverterT, D, Ctx>
where 
    D: Data + Send + Sync + ?Sized,
    Ctx: ContextData + Send + Sync + ?Sized,
    ResponseT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    RequestT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    RequestExecutorT: HandlerExecutor<D, Ctx, RequestT, ResponseT> + Send + Sync,
    RequestConverterT: RequestConverter<Request<Body>, RequestT, HyperBodyConvertError> + 'static + Send + Sync
{
    pub request_executor: Arc<RequestExecutorT>,
    pub converter: Arc<RequestConverterT>,
    pub response: PhantomData<ResponseT>,
    pub request: PhantomData<RequestT>,
    pub d: PhantomData<D>,
    pub ctx: PhantomData<Ctx>
}

impl <RequestT, ResponseT, RequestExecutorT, RequestConverterT, D, Ctx>
HyperRequestStream<RequestT, ResponseT, RequestExecutorT, RequestConverterT, D, Ctx>
where
    D: Data + Send + Sync + ?Sized,
    Ctx: ContextData + Send + Sync + ?Sized,
    ResponseT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    RequestT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    RequestExecutorT: HandlerExecutor<D, Ctx, RequestT, ResponseT> + Send + Sync + 'static,
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
            d: Default::default(),
            ctx: Default::default(),
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

impl <RequestT, ResponseT, RequestExecutorT, RequestConverterT, D, Ctx>
HyperRequestStream<RequestT, ResponseT, RequestExecutorT, RequestConverterT, D, Ctx>
    where
        D: Data + Send + Sync + ?Sized,
        Ctx: ContextData + Send + Sync + ?Sized,
        ResponseT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        RequestT: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        RequestExecutorT: HandlerExecutor<D, Ctx, RequestT, ResponseT> + Send + Sync + 'static,
        RequestConverterT: RequestConverter<Request<Body>, RequestT, HyperBodyConvertError> + 'static + Send + Sync
{

    pub async fn do_run(&mut self) {
        // let addr = ([127, 0, 0, 1], 3000).into();

        // let service = make_service_fn(|cnn: &AddrStream| {
        //     let converter = self.converter.clone();
        //     let request_executor = self.request_executor.clone();
            // async move  {
            //     Ok::<_, Error>(service_fn(move |requested| {
            //         let converter = converter.clone();
            //         let request_executor = request_executor.clone();
            //         async move {
            //             converter.from(requested).await
            //                 .map_err(|e| {
            //                     error!("Error in service function converting from request: {:?}", e);
            //                     e
            //                 })
            //                 .ok()
            //                 .map(|mut converted| {
            //                     let web_response
            //                         = request_executor.execute_handler(converted);
            //                     serde_json::to_string::<ResponseT>(&web_response)
            //                         .map(|res| Response::new(Body::from(res)))
            //                         .map_err(|e| {
            //                             error!("Error in service function converting response to json: {:?}", e);
            //                             HyperRequestStreamError::new("Failed.")
            //                         })
            //                 })
            //                 .or(Some(Err(HyperRequestStreamError::new("Failed to convert request using request converter."))))
            //                 .unwrap()
            //         }
            //     }))
            // }
        // });

        // let server = Server::bind(&addr)
        //     .serve(service);
        // 
        // if let Err(e) = server.await {
        //     error!("server error: {:?}", e);
        // }
    }
}

#[derive(Debug, Default)]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Error as Debug>::fmt(self, f)
    }
}

impl StdError for Error {}

#[derive(Clone, Default)]
pub struct HyperRequestConverter {
    request_extractor: HyperRequestExtractor
}

impl HyperRequestConverter {
    pub fn new() -> Self{
        HyperRequestConverter::default()
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
        let endpoint_metadata = self.request_extractor.convert_extract(&in_value);
        let uri = in_value.uri().clone();
        let method = in_value.method().clone();
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
                        uri,
                        method,
                        endpoint_metadata
                    }
                })
            })
            .or(Err(HyperBodyConvertError::new("Could not convert from Hyper request")))
    }
}

#[derive(Default, Clone)]
pub struct HyperRequestExtractor;

impl RequestTypeExtractor<Request<Body>, EndpointMetadata> for HyperRequestExtractor {
    fn convert_extract(&self, request: &Request<Body>) -> Option<EndpointMetadata> {
        Some(to_endpoint_metadata(request))
    }
}

pub(crate) fn to_endpoint_metadata(request: &Request<Body>) -> EndpointMetadata {
    EndpointMetadata {
        path_variables: split_path_variables(request.uri().path()),
        query_params: request.uri().path_and_query().iter()
            .flat_map(|p| p.query().into_iter())
            .flat_map(|q| split_query_params(q))
            .collect::<HashMap<String, String>>(),
        host: request.uri().host().map(|h| h.to_string())
            .or(Some(String::default())).unwrap(),
    }
}

pub(crate) fn split_path_variables(in_string: &str) -> HashMap<usize, String> {
    let mut c = 0;
    in_string.split("/").into_iter()
        .map(|i| {
            let next = (c, i.to_string());
            c  += 1;
            next
        })
        .collect::<HashMap<usize, String>>()
}

pub(crate) fn split_query_params(in_string: &str) -> HashMap<String, String> {
    in_string.split("&").into_iter()
        .flat_map(|query_item| {
            let mut query_item = query_item.split("=").into_iter()
                .map(|query_items| query_items.to_string())
                .collect::<Vec<String>>();
            if query_item.len() == 0 {
                vec![]
            } else if query_item.len() != 2 {
                error!("Query item found that should have had two parameters, one=two, but was {:?}", &query_item);
                vec![]
            } else {
                let key = query_item.remove(0);
                let value = query_item.remove(0);
                vec![(key, value)]
            }
        }).collect::<HashMap<String, String>>()
}


