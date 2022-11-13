use std::collections::LinkedList;
use crate::convert::{ConverterRegistry, Registration, JsonMessageConverter, OtherMessageConverter};

pub struct Context {
    pub converters: ConverterRegistry
}

impl Default for Context {
    fn default() -> Self {
        let mut registry = ConverterRegistry {
            converters: Box::new(LinkedList::new())
        };
        registry.register(&JsonMessageConverter {});
        registry.register(&OtherMessageConverter {});
        Self {
            converters: registry
        }
    }
}
