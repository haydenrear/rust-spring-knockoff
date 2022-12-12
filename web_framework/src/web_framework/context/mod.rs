mod test;

use core::borrow::BorrowMut;
use crate::web_framework::convert::{ConverterRegistry, EndpointRequestExtractor, JsonMessageConverter, MessageConverter, OtherMessageConverter, Registration, Registry};
use crate::web_framework::security::security::{AuthenticationConverter, AuthenticationConverterRegistry,
                                     AuthenticationToken, AuthenticationType, AuthenticationTypeConverterImpl,
                                     Converter, DelegatingAuthenticationManager};
use crate::web_framework::filter::filter::{FilterChain, RequestResponseActionFilter};
use std::any::Any;
use std::cell::RefCell;
use std::collections::LinkedList;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::web_framework::http::{ProtocolToAdaptFrom, RequestConverter, RequestStream};
use crate::web_framework::request::request::{EndpointMetadata, WebRequest, WebResponse, ResponseWriter};

#[derive(Clone)]
pub struct RequestContext {
    pub message_converters: ConverterRegistry,
    pub authentication_manager: DelegatingAuthenticationManager
}

impl RequestContext {
    pub fn new() -> RequestContext {
        Self {
            message_converters: ConverterRegistry::new(&None),
            authentication_manager: DelegatingAuthenticationManager {providers: LinkedList::new()}
        }
    }
}

impl ContextType<ConverterRegistry, dyn MessageConverter> for RequestContext {
    fn detach_registry(&self) -> ConverterRegistry {
        self.message_converters.clone()
    }
}

pub struct ApplicationContext<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub filter_registry: FilterRegistrar<Request, Response>,
    pub converter_registry: RequestContext,
    pub authentication_converters: AuthenticationConverterRegistry,
    pub auth_type_convert: AuthenticationTypeConverterImpl
}

impl <'a> Registration<'a, dyn MessageConverter> for RequestContext
    where 'a: 'static
{
    fn register(&mut self, converter: &'a dyn MessageConverter) {
        self.message_converters.register(converter);
    }
}

impl <'a, Request, Response> Registration<'a, dyn MessageConverter> for ApplicationContext<Request, Response>
    where 'a: 'static,
          Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
          Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    fn register(&mut self, converter: &'a dyn MessageConverter) {
        self.converter_registry.register(converter)
    }
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
            filter_registry: FilterRegistrar::new(),
            converter_registry: RequestContext::new(),
            authentication_converters: AuthenticationConverterRegistry::new(),
            auth_type_convert: AuthenticationTypeConverterImpl {}
        }
    }

    pub fn with_filter_registry(f: FilterRegistrar<Request, Response>) -> Self {
        Self {
            filter_registry: f,
            converter_registry: RequestContext::new(),
            authentication_converters: AuthenticationConverterRegistry::new(),
            auth_type_convert: AuthenticationTypeConverterImpl {}
        }
    }

    pub fn convert_authentication(&self, request: &WebRequest) -> AuthenticationType {
        self.auth_type_convert.convert(request)
    }

    pub fn extract_authentication(&self, request: &WebRequest) -> AuthenticationToken {
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
                converter_registry: self.converter_registry.clone(),
                authentication_converters: self.authentication_converters.clone(),
                auth_type_convert: self.auth_type_convert.clone()
            }
        }
}

impl <'a, Request, Response> Registration<'a, dyn AuthenticationConverter> for ApplicationContext<Request, Response>
where
    'a : 'static,
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
fn register(&mut self, converter: &'a dyn AuthenticationConverter) {
        self.authentication_converters.register(converter);
    }
}

pub struct FilterContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub registry: FilterRegistrar<Request, Response>,
}

// impl <'a> Registry<dyn Filter> for FilterRegistrar<'a> {
//     fn read_only_registrations(&self) -> Box<LinkedList<&'a dyn Filter>> {
//         Box::new(self.filters.clone())
//     }
// }

pub struct FilterRegistrar<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub filters: Arc<Mutex<Vec<RequestResponseActionFilter<Request, Response>>>>,
    pub filters_build: Arc<FilterChain<Request, Response>>,
    pub build: bool,
}

impl <Request, Response> FilterRegistrar<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{

    pub fn with_filter(&mut self, filter: RequestResponseActionFilter<Request, Response>) {
        self.filters.lock().unwrap().borrow_mut().push(filter);
    }

    // Sets the filter_build for later - so you don't have to do it every time.
    pub fn build(&mut self) -> Arc<FilterChain<Request, Response>> {
        if self.build {
            return self.filters_build.clone();
        }
        let result = self.filters.lock().unwrap();
        let mut filters_found: Vec<RequestResponseActionFilter<Request, Response>> = result.clone();
        filters_found.sort();
        self.filters_build = Arc::new(FilterChain {
            filters: Arc::new(filters_found)
        });
        self.filters_build.clone()
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
            filters_build: self.filters_build.clone(),
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
            filters_build: Arc::new(FilterChain::default()),
            build: false
        }
    }
}

pub trait ContextType<R: Registry<C>, C: ?Sized> {
    fn detach_registry(&self) -> R;
}

impl Default for RequestContext {
    fn default() -> Self {
        let mut registry = ConverterRegistry {
            converters: Box::new(LinkedList::new()),
            request_convert:  Some(&EndpointRequestExtractor {})
        };
        registry.register(&JsonMessageConverter {});
        registry.register(&OtherMessageConverter {});
        Self {
            message_converters: registry,
            authentication_manager: DelegatingAuthenticationManager::new()
        }
    }
}
