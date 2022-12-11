mod test;

use crate::convert::{ConverterRegistry, EndpointRequestExtractor, JsonMessageConverter, MessageConverter, OtherMessageConverter, Registration, Registry};
use crate::filter::filter::{Filter, FilterChain};
use crate::security::security::{AuthenticationConverter, AuthenticationConverterRegistry, AuthenticationToken, AuthenticationType, AuthenticationTypeConverterImpl, Converter, DelegatingAuthenticationManager};
use std::any::Any;
use std::collections::LinkedList;
use serde::{Deserialize, Serialize};
use crate::http::{ProtocolToAdaptFrom, RequestConverter, RequestStream};
use crate::request::request::{EndpointMetadata, WebRequest, WebResponse, ResponseWriter};

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

pub struct ApplicationContext {
    pub filter_registry: FilterRegistrar,
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

impl <'a> Registration<'a, dyn MessageConverter> for ApplicationContext
    where 'a: 'static
{
    fn register(&mut self, converter: &'a dyn MessageConverter) {
        self.converter_registry.register(converter)
    }
}

impl <'a> Registration<'a, dyn Filter> for FilterRegistrar
    where 'a: 'static
{
    fn register(&mut self, converter: &'a dyn Filter) {
        self.filters.push_back(converter.clone())
    }
}

impl <'a> Registration<'a, dyn Filter> for ApplicationContext
    where 'a: 'static
{
    fn register(&mut self, converter: &'a dyn Filter) {
        self.filter_registry.register(converter)
    }
}

impl ApplicationContext {

    /**
    New filter chain for each request - because it's mutable self reference. Because the filter chain
    has lifetime of 'a and it's being added to that, even though filterRegistrar Filter have lifetime of 'static
    it will go to lifetime of 'a, and therefore fix issue of unending static memory. coercion
    */
    pub fn create_get_filter_chain<'a>(&self) -> FilterChain<'a> {
        let filters = self.filter_registry.filters.clone();
        FilterChain::new(filters.iter().map(|v| v.clone()).collect())
    }

    pub fn new() -> Self {
        Self {
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

impl <'a> Registration<'a, dyn AuthenticationConverter> for ApplicationContext
where
    'a : 'static
{
    fn register(&mut self, converter: &'a dyn AuthenticationConverter) {
        self.authentication_converters.register(converter);
    }
}

#[derive(Clone)]
pub struct FilterContext {
    pub registry: FilterRegistrar,
}

impl Registry<dyn Filter> for FilterRegistrar {
    fn read_only_registrations(&self) -> Box<LinkedList<&'static dyn Filter>> {
        Box::new(self.filters.clone())
    }
}

#[derive(Clone)]
pub struct FilterRegistrar {
    pub filters: LinkedList<&'static dyn Filter>,
}

impl FilterRegistrar {
    fn new() -> FilterRegistrar {
        Self {
            filters: LinkedList::new()
        }
    }
}

impl ContextType<FilterRegistrar, dyn Filter> for FilterContext {
    fn detach_registry(&self) -> FilterRegistrar {
        self.registry.clone()
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
