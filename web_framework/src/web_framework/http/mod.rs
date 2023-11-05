use core::slice::Chunks;
use std::collections::LinkedList;
use std::error::Error;
use std::future::Future;
use std::intrinsics::write_bytes;
use std::io::Write;
use std::ops::Deref;
use std::pin::Pin;
use async_std::stream::Stream;
use async_trait::async_trait;
use futures::{FutureExt, StreamExt, TryStream, TryStreamExt};
use crate::web_framework::context::{Context, RequestContextData, RequestHelpers, UserRequestContext};
use serde::{Deserialize, Serialize};
use web_framework_shared::dispatch_server::Handler;
use crate::web_framework::convert::Registration;
use crate::web_framework::dispatch::FilterExecutor;
use web_framework_shared::request::WebResponse;
use web_framework_shared::request::WebRequest;
use crate::web_framework::request_context::SessionContext;
use crate::web_framework::session::session::HttpSession;

#[async_trait]
pub trait RequestStream<'a, RequestResponseType, ResponseWriterType>: Send + Sync
where
    RequestResponseType: Serialize + for<'b> Deserialize<'b> + Clone + Default,
    ResponseWriterType: Copy + Clone
{
    async fn next(&self) -> RequestResponseType;
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

impl <'a> Into<ChunkedBytes> for &[u8] {
    fn into(self) -> ChunkedBytes {
        let mut chunks = vec![];
        let exact = self.chunks_exact(4096);
        let remainder: [u8; 4096] = exact.remainder().try_into().unwrap();
        for chunk in exact.into_iter() {
            let found: [u8; 4096] = chunk.try_into().unwrap();
            chunks.push(found);
        }
        chunks.push(remainder);
        ChunkedBytes {
            bytes: chunks
        }
    }
}

pub struct ChunkedBytes {
    bytes: Vec<[u8; 4096]>
}

