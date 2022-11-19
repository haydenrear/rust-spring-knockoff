use serde::{Deserialize, Serialize};
use crate::context::Context;
use crate::controller::{RequestMethodDispatcher};
use crate::request::request::{EndpointMetadata, HttpRequest};

pub enum HttpMethodAction<'a, Response,Request>
{
    Post(&'a mut dyn RequestMethodDispatcher<Response, Request>, HttpMethod),
    Get(&'a mut dyn RequestMethodDispatcher<Response, Request>, HttpMethod)
}

pub enum HttpMethod
{
    Post, Get
}
