mod test;

use core::borrow::BorrowMut;
use crate::web_framework::convert::{AuthenticationConverterRegistry, ConverterRegistry, MessageConverter, Registration};
use std::any::Any;
use std::{vec};
use std::default::Default;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc};
use serde::{Deserialize, Serialize};
use web_framework_shared::controller::{ContextData, Data};
use web_framework_shared::convert::Converter;
use web_framework_shared::request::EndpointMetadata;
use crate::web_framework::context_builder::{FilterRegistrarBuilder};
use crate::web_framework::http::{RequestConverter, RequestStream};
use crate::web_framework::request_context::SessionContext;
use crate::web_framework::security::authentication::{AuthenticationConverter, DelegatingAuthenticationManager};



pub struct RequestContextData<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub request_context_data: Context<Request, Response>
}

impl<Request, Response> Data for RequestContextData<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{}

impl <Request, Response> ContextData for RequestContextData<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {}

#[derive(Default, Clone)]
pub struct UserRequestContext<Request>
    where
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub request_context: SessionContext,
    pub request: Option<Request>,
    pub endpoint_metadata: Option<EndpointMetadata>
}

impl<Request>  UserRequestContext<Request>
where
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub fn new_default() -> Self {
        Self {
            request_context: SessionContext::default(),
            request: None,
            endpoint_metadata: Some(EndpointMetadata::default())
        }
    }
}

impl<Request> ContextData for UserRequestContext<Request>
where
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{}

impl <Request> Data for UserRequestContext<Request>
    where
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static{}

#[derive(Clone, Default)]
pub struct RequestHelpers<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub message_converters: ConverterRegistry<Request, Response>,
    pub authentication_manager: DelegatingAuthenticationManager
}

impl <Request, Response> RequestHelpers<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub fn new() -> RequestHelpers<Request, Response> {
        Self {
            message_converters: ConverterRegistry::new(None, None),
            authentication_manager: DelegatingAuthenticationManager { providers: Arc::new(vec![]) }
        }
    }

    pub fn with_converter_registry(message_converters: ConverterRegistry<Request, Response>) -> RequestHelpers<Request, Response> {
        Self {
            message_converters,
            authentication_manager: DelegatingAuthenticationManager { providers: Arc::new(vec![]) }
        }
    }
}

pub struct Context<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub filter_registry: Arc<FilterRegistrarBuilder<Request, Response>>,
    pub request_context: RequestHelpers<Request, Response>,
    pub authentication_converters: AuthenticationConverterRegistry,
}

impl <Request, Response> Context<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{

    pub fn new() -> Self {
        Self {
            filter_registry: Arc::new(FilterRegistrarBuilder::new()),
            request_context: RequestHelpers::new(),
            authentication_converters: AuthenticationConverterRegistry::new(),
        }
    }

    pub fn with_converter_registry(message_converters: ConverterRegistry<Request, Response>) -> Self {
        Self {
            filter_registry: Arc::new(FilterRegistrarBuilder::new()),
            request_context: RequestHelpers::with_converter_registry(message_converters),
            authentication_converters: AuthenticationConverterRegistry::new(),
        }
    }

}

impl <'a, Request, Response> Clone for Context<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    fn clone(&self) -> Self {
            Self {
                filter_registry: self.filter_registry.clone(),
                request_context: self.request_context.clone(),
                authentication_converters: self.authentication_converters.clone(),
            }
        }
}

pub struct FilterContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub registry: FilterRegistrarBuilder<Request, Response>,
}
