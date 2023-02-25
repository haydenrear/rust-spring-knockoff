mod test;

use core::borrow::BorrowMut;
use crate::web_framework::convert::{AuthenticationConverterRegistry, ConverterRegistry, EndpointRequestExtractor, JsonMessageConverter, MessageConverter, OtherMessageConverter, Registration};
use crate::web_framework::security::security::{AuthenticationConverter, AuthenticationToken, DelegatingAuthenticationManager};
use crate::web_framework::filter::filter::{FilterChain, RequestResponseActionFilter};
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
use crate::web_framework::context_builder::{AuthenticationConverterRegistryBuilder, ConverterRegistryBuilder, DelegatingAuthenticationManagerBuilder};
use crate::web_framework::dispatch::Dispatcher;
use crate::web_framework::http::{ProtocolToAdaptFrom, RequestConverter, RequestStream};

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
            message_converters: ConverterRegistry::new(None),
            authentication_manager: DelegatingAuthenticationManager {providers: Arc::new(vec![])}
        }
    }
}

pub struct ApplicationContext<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub filter_registry: Arc<FilterRegistrar<Request, Response>>,
    pub request_context: RequestContext<Request, Response>,
    pub authentication_converters: AuthenticationConverterRegistry,
    pub auth_type_convert: AuthenticationTypeConverterImpl
}

impl <Request, Response> FilterRegistrar<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub fn register(&mut self, converter: RequestResponseActionFilter<Request, Response>) {
        self.filters.lock().unwrap().borrow_mut().push(converter)
    }
}

impl <Request, Response> ApplicationContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{

    pub fn new() -> Self {
        Self {
            filter_registry: Arc::new(FilterRegistrar::new()),
            request_context: RequestContext::new(),
            authentication_converters: AuthenticationConverterRegistry::new(),
            auth_type_convert: AuthenticationTypeConverterImpl::new()
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
                auth_type_convert: self.auth_type_convert.clone()
            }
        }
}

pub struct FilterContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub registry: FilterRegistrar<Request, Response>,
}

pub struct FilterRegistrar<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub filters: Arc<Mutex<Vec<RequestResponseActionFilter<Request, Response>>>>,
    pub fiter_chain: Arc<FilterChain<Request, Response>>,
    pub build: bool,
}

impl <Request, Response> FilterRegistrar<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{

    // Sets the filter_build for later - so you don't have to do it every time.
    pub fn build(&mut self) -> Arc<FilterChain<Request, Response>> {
        if self.build {
            return self.fiter_chain.clone();
        }
        let result = self.filters.lock().unwrap();
        let mut filters_found: Vec<RequestResponseActionFilter<Request, Response>> = result.clone();
        filters_found.sort();
        self.fiter_chain = Arc::new(FilterChain {
            filters: Arc::new(filters_found)
        });
        self.fiter_chain.clone()
    }
}

impl <Request, Response> Clone for FilterRegistrar<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync

{
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone(),
            fiter_chain: self.fiter_chain.clone(),
            build: self.build
        }
    }
}

impl <Request, Response> FilterRegistrar<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub(crate) fn new() -> FilterRegistrar<Request, Response> {
        Self {
            filters: Arc::new(Mutex::new(vec![])),
            fiter_chain: Arc::new(FilterChain::default()),
            build: false
        }
    }
}
