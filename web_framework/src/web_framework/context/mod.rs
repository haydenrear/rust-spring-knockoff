mod test;

use core::borrow::BorrowMut;
use crate::web_framework::convert::{ConverterRegistry, ConverterRegistryBuilder, EndpointRequestExtractor, JsonMessageConverter, MessageConverter, OtherMessageConverter, Registration};
use crate::web_framework::security::security::{AuthenticationConversionError, AuthenticationConverter, AuthenticationConverterRegistry, AuthenticationConverterRegistryBuilder, AuthenticationToken, AuthenticationType, AuthenticationTypeConverterImpl, Converter, DelegatingAuthenticationManager, DelegatingAuthenticationManagerBuilder};
use crate::web_framework::filter::filter::{FilterChain, RequestResponseActionFilter};
use std::any::Any;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::marker::PhantomData;
use std::{mem, vec};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::web_framework::dispatch::Dispatcher;
use crate::web_framework::http::{ProtocolToAdaptFrom, RequestConverter, RequestStream};
use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse, ResponseWriter};

#[derive(Clone, Default)]
pub struct RequestContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub message_converters: ConverterRegistry<Request, Response>,
    pub authentication_manager: DelegatingAuthenticationManager
}

#[derive(Clone)]
pub struct RequestContextBuilder <Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub message_converter_builder: ConverterRegistryBuilder<Request, Response>,
    pub authentication_manager_builder: DelegatingAuthenticationManagerBuilder
}

impl <Request, Response> RequestContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    fn build(&mut self) -> RequestContext<Request, Response> {
        RequestContext {
            message_converters: self.message_converter_builder.build(),
            authentication_manager: self.authentication_manager_builder.build(),
        }
    }
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

pub struct ApplicationContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub filter_registry: Option<Arc<Mutex<FilterRegistrar<Request, Response>>>>,
    pub request_context_builder: Option<Arc<Mutex<RequestContextBuilder<Request, Response>>>>,
    pub authentication_converters: Option<Arc<AuthenticationConverterRegistryBuilder>>
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

// impl <'a> Registration<'a, dyn Filter> for ApplicationContext
// {
//     fn register(&mut self, converter: &'a dyn Filter) {
//         self.filter_registry.register(converter)
//     }
// }

impl <Request, Response> ApplicationContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub fn build(&self) -> ApplicationContext<Request, Response> {
        let mut filter_registry_found = self.filter_registry.as_ref().unwrap().lock().unwrap().clone();
        filter_registry_found.build();
        let context = self.request_context_builder.as_ref().unwrap().lock().unwrap().build();
        ApplicationContext {
            filter_registry: Arc::new(filter_registry_found),
            request_context: context,
            authentication_converters: self.authentication_converters.as_ref().unwrap().build(),
            auth_type_convert: AuthenticationTypeConverterImpl,
        }
    }
}

impl <Request, Response> ApplicationContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{

    /**
    New filter chain for each request - because it's mutable self reference. Because the filter chain
    has lifetime of 'a and it's being added to that, even though filterRegistrar Filter have lifetime of 'static
    it will go to lifetime of 'a, and therefore fix issue of unending static memory. coercion
    */
    // pub fn create_get_filter_chain(&self) -> FilterChain<Request, Response> {
    //     if self.filter_chain.is_some() {
    //         self.filter_chain
    //     }
    //     let vec = self.filter_registry.filters.lock().unwrap().clone();
    //     FilterChain::new(vec)
    // }

    pub fn new() -> Self {
        Self {
            filter_registry: Arc::new(FilterRegistrar::new()),
            request_context: RequestContext::new(),
            authentication_converters: AuthenticationConverterRegistry::new(),
            auth_type_convert: AuthenticationTypeConverterImpl {}
        }
    }

    pub fn with_filter_registry(f: FilterRegistrar<Request, Response>) -> Self {
        Self {
            filter_registry: Arc::new(f),
            request_context: RequestContext::new(),
            authentication_converters: AuthenticationConverterRegistry::new(),
            auth_type_convert: AuthenticationTypeConverterImpl {}
        }
    }

    pub fn and_converter_registry(&mut self) {

    }

    pub fn convert_authentication(&self, request: &WebRequest) -> Result<AuthenticationType, AuthenticationConversionError> {
        self.auth_type_convert.convert(request)
    }

    pub fn extract_authentication(&self, request: &WebRequest) -> Result<AuthenticationToken, AuthenticationConversionError> {
        self.authentication_converters.convert(request)
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

impl <'a, Request, Response> Registration<'a, dyn AuthenticationConverter> for ApplicationContextBuilder<Request, Response>
where
    'a : 'static,
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
fn register(&self, converter: &'a dyn AuthenticationConverter) {
        self.authentication_converters.as_ref()
            .map(|r| {
                r.register(converter)
            });
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
    fn new() -> FilterRegistrar<Request, Response> {
        Self {
            filters: Arc::new(Mutex::new(vec![])),
            fiter_chain: Arc::new(FilterChain::default()),
            build: false
        }
    }
}

impl <Request, Response> Default for RequestContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    fn default() -> Self {
        let mut registry = ConverterRegistryBuilder {
            converters: Arc::new(Mutex::new(Some(Box::new(OtherMessageConverter{})))),
            request_convert:  Arc::new(Mutex::new(Some(&EndpointRequestExtractor { })))
        };
        Self {
            message_converter_builder: registry,
            authentication_manager_builder: DelegatingAuthenticationManagerBuilder {
                providers: Arc::new(Mutex::new(vec![]))
            },
        }
    }
}
