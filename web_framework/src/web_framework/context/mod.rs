mod test;

use core::borrow::BorrowMut;
use crate::web_framework::convert::{AuthenticationConverterRegistry, ConverterRegistry, DefaultMessageConverter, EndpointRequestExtractor, MessageConverter, Registration};
use crate::web_framework::filter::filter::{DelegatingFilterProxy, Filter};
use std::any::Any;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::marker::PhantomData;
use std::{mem, vec};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use module_macro_lib::{AuthenticationType, AuthenticationTypeConverterImpl};
use web_framework_shared::convert::Converter;
use crate::web_framework::context_builder::{AuthenticationConverterRegistryBuilder, ConverterRegistryBuilder, DelegatingAuthenticationManagerBuilder, FilterRegistrarBuilder};
use crate::web_framework::dispatch::Dispatcher;
use crate::web_framework::http::{ProtocolToAdaptFrom, RequestConverter, RequestStream};
use crate::web_framework::security::authentication::{AuthenticationConverter, AuthenticationToken, DelegatingAuthenticationManager};

#[derive(Clone, Default)]
pub struct RequestContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub message_converters: ConverterRegistry<Request, Response>,
    pub authentication_manager: DelegatingAuthenticationManager
}

impl <Request, Response> RequestContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub fn new() -> RequestContext<Request, Response> {
        Self {
            message_converters: ConverterRegistry::new(None, None),
            authentication_manager: DelegatingAuthenticationManager {providers: Arc::new(vec![])}
        }
    }
}

pub struct ApplicationContext<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub filter_registry: Arc<FilterRegistrarBuilder<Request, Response>>,
    pub request_context: RequestContext<Request, Response>,
    pub authentication_converters: AuthenticationConverterRegistry,
}

impl <Request, Response> ApplicationContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{

    pub fn new() -> Self {
        Self {
            filter_registry: Arc::new(FilterRegistrarBuilder::new()),
            request_context: RequestContext::new(),
            authentication_converters: AuthenticationConverterRegistry::new(),
        }
    }

}

impl <'a, Request, Response> Clone for ApplicationContext<Request, Response>
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
