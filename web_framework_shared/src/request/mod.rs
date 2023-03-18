use std::collections::{HashMap, LinkedList};
use std::fs::Metadata;
use std::io::{Bytes, Read, Write};
use std::marker::PhantomData;
use std::net::TcpStream;
use std::ops::Deref;
use async_std::io::ReadExt;
use crate::http_method::HttpMethod;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use circular::Buffer;

//TODO: turn into uri components builder
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct EndpointMetadata {
    pub path_variables: String,
    pub query_params: HashMap<String, String>,
    pub http_method: HttpMethod,
    pub base_uri: String
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct WebRequest {
    pub headers: HashMap<String, String>,
    pub body: String,
    pub metadata: EndpointMetadata,
    pub method: HttpMethod,
    pub uri: String
}

pub trait AuthorizationObject: Send + Sync + Default + Clone {
}

impl AuthorizationObject for WebRequest {}

trait HttpEntity {}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WebResponse {
    pub response: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub response_bytes: ResponseBytesBuffer
}

impl WebResponse {
    pub fn response_bytes(&mut self) -> Result<Vec<u8>, String> {
        self.response_bytes.read_and_empty_buffer()
    }
}

#[derive(Clone)]
pub struct ResponseBytesBuffer {
    response_bytes: Buffer
}

impl ResponseBytesBuffer {
    const SIZE: usize = 4096;
    fn add_bytes(&mut self, mut bytes: &[u8]) {
        self.response_bytes.write(bytes);
    }

    fn next(&mut self) -> [u8; Self::SIZE] {
        let mut more_bytes: [u8; Self::SIZE] = [0; Self::SIZE];
        self.response_bytes.read(more_bytes.as_mut());
        more_bytes
    }

    fn empty(&self) -> bool {
        self.response_bytes.empty()
    }

    pub fn write(&mut self, to_write: &[u8]) -> std::io::Result<usize> {
        if self.response_bytes.available_space() < to_write.len()  {
            self.response_bytes.grow(self.response_bytes.capacity() + to_write.len() * 2);
        }
        self.response_bytes.write(to_write)
    }

    pub fn read_and_empty_buffer(&mut self) -> Result<Vec<u8>, String> {
        let mut created_vec = vec![0; self.response_bytes.available_data()];
        let mut response: &mut [u8] = created_vec.as_mut_slice();
        self.response_bytes.read(response)
            .map(|r| created_vec)
            .or(Err("Error reading from buffer".to_string()))
    }

}

impl Default for ResponseBytesBuffer {
    fn default() -> Self {
        Self {
            response_bytes: Buffer::with_capacity(12000)
        }
    }
}

pub trait ResponseWriter<T> {
    fn write(&mut self, response: T);
}

impl <'a> ResponseWriter<&[u8]> for WebResponse {
    fn write(&mut self, response: &[u8]) {
        self.response = String::from_utf8(Vec::from(response))
            .map(|response_str| self.response.clone() + response_str.as_str())
            .unwrap_or(self.response.clone());
        self.response_bytes.write(response);
    }
}


