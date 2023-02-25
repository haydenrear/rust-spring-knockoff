use std::collections::{HashMap, LinkedList};
use std::fs::Metadata;
use std::io::{Bytes, Read, Write};
use std::marker::PhantomData;
use std::net::TcpStream;
use std::ops::Deref;
use async_std::io::ReadExt;
use crate::http_method::HttpMethod;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct EndpointMetadata {
    pub path_variables: String,
    pub query_params: String,
    pub http_method: HttpMethod
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct WebRequest {
    pub headers: HashMap<String, String>,
    pub body: String,
    pub metadata: EndpointMetadata,
    pub method: HttpMethod,
}

