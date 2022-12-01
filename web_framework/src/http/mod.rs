use async_trait::async_trait;
use crate::context::RequestContext;
use crate::controller::RequestMethodDispatcher;
use crate::request::request::{EndpointMetadata, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::dispatch::RequestMethodDispatcher;

pub enum HttpMethod {
    Post,
    Get,
}
