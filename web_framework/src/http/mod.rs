use core::slice::Chunks;
use std::collections::LinkedList;
use std::future::Future;
use std::intrinsics::write_bytes;
use std::io::Write;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};
use async_std::stream::{Stream};
use async_trait::async_trait;
use futures::{FutureExt, StreamExt, TryStream, TryStreamExt};
use crate::context::RequestContext;
use crate::request::request::{EndpointMetadata, HttpRequest, HttpResponse, ResponseWriter};
use serde::{Deserialize, Serialize};
use crate::dispatch::{Dispatcher, RequestMethodDispatcher};
use crate::filter::filter::Action;

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
    RequestResponseItem: Serialize + for<'b> Deserialize<'b> + Clone + Default + ResponseWriter<ResponseWriterType>,
    ResponseWriterType: Copy + Clone
{
    fn subscribe(&self) -> &RequestResponseStream;
}

#[async_trait]
pub trait NextFuture<RequestResponseType, ResponseWriterType>
where
    RequestResponseType: Serialize + for<'b> Deserialize<'b> + Clone + Default + ResponseWriter<ResponseWriterType>,
    ResponseWriterType: Copy + Clone
{
    async fn next(&self) -> RequestResponseType;
}

#[async_trait]
pub trait RequestStream<'a, RequestResponseType, ResponseWriterType>: NextFuture<RequestResponseType, ResponseWriterType>
where
    RequestResponseType: Serialize + for<'b> Deserialize<'b> + Clone + Default + ResponseWriter<ResponseWriterType>,
    ResponseWriterType: Copy + Clone
{
    fn flush_response(& self, response_writer_type: &'a mut RequestResponseType);
}

pub trait RequestConverter<T, U>: Send + Sync
where
    U: Serialize + for<'b> Deserialize<'b> + Clone + Default
{
    fn from(&self, in_value: T) -> U;
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
pub struct TestResponseType<'a> {
    #[serde(skip_serializing, skip_deserializing)]
    pub connection: Option<&'a dyn Connection<'a, &'a [u8]>>,
    pub response: HttpResponse
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TestRequestType<'a> {
    #[serde(skip_serializing, skip_deserializing)]
    pub connection: Option<&'a dyn Connection<'a, &'a [u8]>>,
    request: HttpRequest,
    response: HttpResponse
}

impl <'a> Default for TestRequestType<'a> {
    fn default() -> Self {
        todo!()
    }
}

impl <'a> Default for TestResponseType<'a> {
    fn default() -> Self {
        todo!()
    }
}

impl <'a> ResponseWriter<&[u8]> for TestResponseType<'a> {
    fn write(&mut self, response: &[u8]) {
        self.response.write_to_cxn(self.connection.unwrap())
    }
}

impl <'a> ResponseWriter<&[u8]> for TestRequestType<'a> {
    fn write(&mut self, response: &[u8]) {
        self.response.write_to_cxn(self.connection.unwrap())
    }
}

pub struct RequestStreamImpl<'a, RequestResponseItem, ResponseWriterType, Response>
    where
        RequestResponseItem: Serialize + for<'b> Deserialize<'b> + Clone + Default + ResponseWriter<ResponseWriterType>,
        ResponseWriterType: Copy + Clone,
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + ResponseWriter<ResponseWriterType>,
        Self: 'a
{
    protocol: &'a dyn ProtocolToAdaptFrom<'a, Self, RequestResponseItem, RequestResponseItem>,
    converter: &'a dyn RequestConverter<RequestResponseItem, Response>
}

#[async_trait]
impl <'a> NextFuture<TestRequestType<'a>, &'a [u8]> for RequestStreamImpl<'a, TestRequestType<'a>, &'a [u8], TestResponseType<'a>> {
    async fn next(&self) -> TestRequestType<'a> {
        todo!()
    }
}

impl <'a> RequestStream<'a, TestRequestType<'a>, &'a [u8]> for RequestStreamImpl<'a, TestRequestType<'a>, &'a [u8], TestResponseType<'a>>
{
    fn flush_response(&self, response_writer_type: & mut TestRequestType<'a>) {
        let mut response = self.converter.from(response_writer_type.clone());
        response_writer_type.connection
            .map(|cxn| {
                response.response.write_to_cxn(cxn);
            });
    }
}
