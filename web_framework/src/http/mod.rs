use core::slice::Chunks;
use std::async_iter::AsyncIterator;
use std::collections::LinkedList;
use std::error::Error;
use std::future::Future;
use std::intrinsics::write_bytes;
use std::io::Write;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};
use async_std::stream::{Stream};
use async_trait::async_trait;
use futures::{FutureExt, StreamExt, TryStream, TryStreamExt};
use crate::context::{ApplicationContext, RequestContext};
use crate::request::request::{EndpointMetadata, WebRequest, WebResponse, ResponseWriter};
use serde::{Deserialize, Serialize};
use crate::convert::Registration;
use crate::dispatch::{Dispatcher, RequestMethodDispatcher};
use crate::filter::filter::{Action, Filter};

#[derive(Serialize, Deserialize)]
pub enum HttpMethod {
    Post,
    Get,
}

pub trait Adapter<T,U> {
    fn from(&self, t: T) -> U;
}

pub trait ProtocolToAdaptFrom<'a, RequestResponseStream, RequestResponseItem, ResponseWriterType>: Send + Sync
where
    RequestResponseStream: RequestStream<'a, RequestResponseItem, ResponseWriterType>,
    RequestResponseItem: Serialize + for<'b> Deserialize<'b> + Clone + Default,
    ResponseWriterType: Copy + Clone
{
    fn subscribe(&self) -> &RequestResponseStream;
}

#[async_trait]
pub trait RequestStream<'a, RequestResponseType, ResponseWriterType>: Send + Sync
where
    RequestResponseType: Serialize + for<'b> Deserialize<'b> + Clone + Default,
    ResponseWriterType: Copy + Clone
{
    async fn next(&self) -> RequestResponseType;
}

#[async_trait]
pub trait RequestExecutor<'a, RequestType, ResponseType, ResponseWriterType>
    where
        RequestType: Serialize + for<'b> Deserialize<'b> + Clone + Default,
        ResponseType: Serialize + for<'b> Deserialize<'b> + Clone + Default,
        ResponseWriterType: Copy + Clone
{
    fn do_request(&self, response_writer_type: RequestType) -> ResponseType;
}

#[async_trait]
pub trait RequestConverter<T, U, E>: Send + Sync + Clone
where
    U: Serialize + for<'b> Deserialize<'b> + Clone + Default,
    E: RequestConversionError
{
    async fn from(&self, in_value: T) -> Result<U, E>;
}

pub trait RequestConversionError: Error {

}

pub trait Connection<'a, RequestResponseItem>: Send + Sync
where
    RequestResponseItem: Into<ChunkedBytes>,
{
    fn write_bytes(&self, to_write: [u8; 4096]);
    fn write(&self, to_write: RequestResponseItem) {
        to_write.into().bytes.iter().for_each(|bytes| {
            self.write_bytes(*bytes)
        });
    }
}

impl <'a> Into<ChunkedBytes> for &[u8] {
    fn into(self) -> ChunkedBytes {
        let mut chunks = LinkedList::new();
        let exact = self.chunks_exact(4096);
        let remainder: [u8; 4096] = exact.remainder().try_into().unwrap();
        for chunk in exact.into_iter() {
            let found: [u8; 4096] = chunk.try_into().unwrap();
            chunks.push_back(found);
        }
        chunks.push_back(remainder);
        ChunkedBytes {
            bytes: chunks
        }
    }
}

pub struct ChunkedBytes {
    bytes: LinkedList<[u8; 4096]>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResponseType<'a> {
    #[serde(skip_serializing, skip_deserializing)]
    pub connection: Option<&'a dyn Connection<'a, &'a [u8]>>,
    pub response: WebResponse
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RequestType<'a> {
    #[serde(skip_serializing, skip_deserializing)]
    pub connection: Option<&'a dyn Connection<'a, &'a [u8]>>,
    request: WebRequest,
    response: WebResponse
}

impl <'a> Default for RequestType<'a> {
    fn default() -> Self {
        todo!()
    }
}

impl <'a> Default for ResponseType<'a> {
    fn default() -> Self {
        todo!()
    }
}

pub struct RequestExecutorImpl
{
    pub ctx: ApplicationContext
}


#[async_trait]
impl <'a> RequestExecutor<'a, WebRequest, WebResponse, &'a [u8]>
for RequestExecutorImpl
where
{

    fn do_request(&self, mut web_request: WebRequest) -> WebResponse {
        let mut response = WebResponse::default();
        self.ctx.create_get_filter_chain()
            .do_filter(&web_request, &mut response, &self.ctx);
        response
    }
}