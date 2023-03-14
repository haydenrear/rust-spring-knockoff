mod test;

use core::borrow::BorrowMut;
use crate::web_framework::convert::{AuthenticationConverterRegistry, ConverterRegistry, DefaultMessageConverter, EndpointRequestExtractor, MessageConverter, Registration};
use crate::web_framework::filter::filter::{FilterChain, Filter};
use std::any::Any;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::marker::PhantomData;
use std::{mem, vec};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use authentication_gen::{AuthenticationType, AuthenticationTypeConverterImpl};
use web_framework_shared::convert::Converter;
use crate::web_framework::context_builder::{AuthenticationConverterRegistryBuilder, ConverterRegistryBuilder, DelegatingAuthenticationManagerBuilder, FilterRegistrarBuilder};
use crate::web_framework::dispatch::FilterExecutor;
use crate::web_framework::http::{ProtocolToAdaptFrom, RequestConverter, RequestStream};
use crate::web_framework::security::authentication::{AuthenticationConverter, AuthenticationToken, DelegatingAuthenticationManager};

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
