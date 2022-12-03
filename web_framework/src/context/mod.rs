use crate::convert::{ConverterRegistry, EndpointRequestExtractor, JsonMessageConverter, MessageConverter, OtherMessageConverter, Registration, Registry};
use crate::filter::filter::{Filter, FilterChain};
use crate::security::security::AuthenticationConverterRegistry;
use std::any::Any;
use std::collections::LinkedList;
use serde::{Deserialize, Serialize};
use crate::http::{ProtocolToAdaptFrom, RequestConverter, RequestStream};
use crate::request::request::{EndpointMetadata, HttpRequest, HttpResponse, ResponseWriter};

pub struct RequestContext {
    pub message_converters: ConverterRegistry,
}

impl ContextType<ConverterRegistry, dyn MessageConverter> for RequestContext {
    fn detach_registry(&self) -> ConverterRegistry {
        self.message_converters.clone()
    }
}

pub struct ApplicationContext {
    pub filter_registry: FilterRegistrar,
    pub converter_registry: RequestContext,
    pub authentication_converters: AuthenticationConverterRegistry
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
        self.register(converter);
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

}

// impl <RequestStream, Response, IAdaptFrom, ResponseWriterType> Registry<dyn HandlerAdapter<IAdaptFrom, RequestStream, Response, ResponseWriterType>> for ApplicationContext
//     where
//         Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + ResponseWriter<ResponseWriterType>,
//         RequestStream: Serialize + for<'b> Deserialize<'b> + Clone + Default + RequestStream<Response, ResponseWriterType>,
//         ResponseWriterType: Copy + Clone,
//         IAdaptFrom: ProtocolToAdaptFrom<RequestStream, Response, ResponseWriterType>
// {
//     fn read_only_registrations(&self) -> Box<LinkedList<&'static dyn HandlerAdapter<IAdaptFrom, RequestStream, Response, ResponseWriterType>>> {
//         todo!()
//     }
// }

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
            request_convert:  &EndpointRequestExtractor {}
        };
        registry.register(&JsonMessageConverter {});
        registry.register(&OtherMessageConverter {});
        Self {
            message_converters: registry,
        }
    }
}
