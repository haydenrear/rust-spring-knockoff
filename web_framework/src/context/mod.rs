use std::any::Any;
use std::collections::LinkedList;
use crate::convert::{ConverterRegistry, Registration, JsonMessageConverter, OtherMessageConverter, MessageConverter, Registry};
use crate::filter::filter::{Filter, FilterChain};
use crate::security::security::AuthenticationConverterRegistry;

pub struct RequestContext {
    pub message_converters: ConverterRegistry,
}

impl ContextType<ConverterRegistry, dyn MessageConverter> for RequestContext {
    fn detach_registry(&self) -> ConverterRegistry {
        self.message_converters.clone()
    }
}

#[derive(Clone)]
pub struct ApplicationContext {
    pub filter_registry: FilterRegistrar,
    pub converter_registry: ConverterRegistry,
    pub authentication_converters: AuthenticationConverterRegistry,
}

impl ApplicationContext {
    fn add_filter(&mut self, filter: &'static dyn Filter) {
        self.filter_registry.filters.push_back(filter.clone())
    }

    /**
    New filter chain for each request - because it's mutable self reference. **maybe** because the filter chain
    has lifetime of 'a and it's being added to that, even though filterRegistrar Filter have lifetime of 'static
    it will go to lifetime of 'a, and therefore fix issue of unending static memory.
    */
    fn create_get_filter_chain<'a>(&self) -> FilterChain<'a> {
        let filters = self.filter_registry.filters.clone();
        FilterChain::new(filters.iter().map(|v| v.clone()).collect())
    }

}

#[derive(Clone)]
pub struct FilterContext {
    pub registry: FilterRegistrar
}

impl Registry<dyn Filter> for FilterRegistrar {
    fn read_only_registrations(&self) -> Box<LinkedList<&'static dyn Filter>> {
        Box::new(self.filters.clone())
    }
}

#[derive(Clone)]
pub struct FilterRegistrar {
    pub filters: LinkedList<&'static dyn Filter>
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
            converters: Box::new(LinkedList::new())
        };
        registry.register(&JsonMessageConverter {});
        registry.register(&OtherMessageConverter {});
        Self {
            message_converters: registry
        }
    }
}