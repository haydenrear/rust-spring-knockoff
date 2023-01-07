mod test;

use crate::web_framework::convert::{ConverterRegistry, EndpointRequestExtractor, JsonMessageConverter, MessageConverter, OtherMessageConverter, Registration, Registry};
use crate::web_framework::security::security::{AuthenticationConverter, AuthenticationConverterRegistry,
                                     AuthenticationToken, AuthenticationType, AuthenticationTypeConverterImpl,
                                     Converter, DelegatingAuthenticationManager};
use crate::web_framework::filter::filter::{Filter, FilterChain};
use std::any::Any;
use std::collections::LinkedList;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
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

pub struct ApplicationContext<'a> where 'a: 'static {
    pub values_handler: Values,
    pub filter_registry: FilterRegistrar<'a>,
    pub converter_registry: RequestContext,
    pub authentication_converters: AuthenticationConverterRegistry,
    pub auth_type_convert: AuthenticationTypeConverterImpl
}

pub struct Values {
    pub filters: Vec<Option<Box<dyn Filter>>>
}

impl <'a> Registration<'a, dyn MessageConverter> for RequestContext
    where 'a: 'static
{
    fn register(&mut self, converter: &'a dyn MessageConverter) {
        self.message_converters.register(converter);
    }
}

impl <'a> Registration<'a, dyn MessageConverter> for ApplicationContext<'a>
    where 'a: 'static
{
    fn register(&mut self, converter: &'a dyn MessageConverter) {
        self.converter_registry.register(converter)
    }
}

impl <'a> FilterRegistrar<'a> where Self: 'static, 'a: 'static
{
    pub fn register(&mut self, converter: &'a Option<Box<dyn Filter>>) {
        self.filters.push(converter)
    }
}

// impl <'a> Registration<'a, dyn Filter> for ApplicationContext
// {
//     fn register(&mut self, converter: &'a dyn Filter) {
//         self.filter_registry.register(converter)
//     }
// }

impl <'a> ApplicationContext<'a> where 'a: 'static{

    /**
    New filter chain for each request - because it's mutable self reference. Because the filter chain
    has lifetime of 'a and it's being added to that, even though filterRegistrar Filter have lifetime of 'static
    it will go to lifetime of 'a, and therefore fix issue of unending static memory. coercion
    */
    pub fn create_get_filter_chain(&self) -> FilterChain {
        let vec = self.filter_registry.filters.clone();
        FilterChain::new(vec)
    }

    pub fn initialize(&'a mut self) {
        self.values_handler.filters
            .iter_mut()
            .for_each(|filter| {
                self.filter_registry.register(filter)
            })
    }

    pub fn new() -> Self {
        Self {
            values_handler: Values {filters: vec![]},
            filter_registry: FilterRegistrar::new(),
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

impl <'a> Clone for ApplicationContext<'a> {
    fn clone(&self) -> Self {
        Self {
            values_handler: Values {filters: vec![]},
            filter_registry: self.filter_registry.clone(),
            converter_registry: self.converter_registry.clone(),
            authentication_converters: self.authentication_converters.clone(),
            auth_type_convert: self.auth_type_convert.clone()
        }
    }
}

impl <'a> Registration<'a, dyn AuthenticationConverter> for ApplicationContext<'a>
where
    'a : 'static
{
    fn register(&mut self, converter: &'a dyn AuthenticationConverter) {
        self.authentication_converters.register(converter);
    }
}

pub struct FilterContext<'a> {
    pub registry: FilterRegistrar<'a>,
}

// impl <'a> Registry<dyn Filter> for FilterRegistrar<'a> {
//     fn read_only_registrations(&self) -> Box<LinkedList<&'a dyn Filter>> {
//         Box::new(self.filters.clone())
//     }
// }

pub struct FilterRegistrar<'a> {
    pub filters: Vec<&'a Option<Box<dyn Filter>>>
}

impl <'a> Clone for FilterRegistrar<'a> {
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone()
        }
    }
}

impl <'a> FilterRegistrar<'a> {
    fn new() -> FilterRegistrar<'a> {
        Self {
            filters: vec![]
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
