pub mod test;
pub mod filter {

    extern crate alloc;
    extern crate core;

    use crate::request::request::{EndpointMetadata, HttpRequest, HttpResponse};
    use crate::session::session::HttpSession;
    use alloc::string::String;
    use core::borrow::{Borrow, BorrowMut};
    use serde::{Deserialize, Serialize};
    use std::collections::{HashMap, LinkedList};
    use std::path::Iter;

    #[derive(Clone)]
    pub struct FilterChain<'a> {
        filters: Vec<&'a dyn Filter>,
        pub(crate) num: usize,
    }

    impl<'a> FilterChain<'a> {
        pub fn do_filter(&mut self, request: HttpRequest, response: HttpResponse) {
            let next = self.next();
            if next != -1 {
                let f = &self.filters[(next - 1) as usize];
                f.filter(request, response, self.clone());
                if self.num >= self.filters.len() {
                    self.num = 0;
                }
            }
        }

        pub(crate) fn next(&mut self) -> i64 {
            if self.filters.len() > self.num {
                self.num += 1;
                return self.num as i64;
            } else {
                -1
            }
        }

        pub fn new(filters: Vec<&'a dyn Filter>) -> Self {
            Self {
                filters: filters,
                num: 0,
            }
        }
    }

    pub trait Registration<'a, C: MessageConverter> {
        fn register(&mut self, converter: &'a C);
    }

    pub struct ConverterRegistry {
        pub(crate) converters: Box<LinkedList<&'static dyn MessageConverter>>
    }

    impl <'a> Registration<'a, JsonMessageConverter> for ConverterRegistry where 'a: 'static {
        fn register(&mut self, converter: &'a JsonMessageConverter) {
            self.converters.push_front(converter)
        }
    }

    impl Converters for ConverterRegistry {
        fn converters(&self, request: &HttpRequest) -> Box<dyn Iterator<Item=&'static dyn MessageConverter>> {
            Box::new(self.converters.iter()
                .filter(|&c| {
                    c.do_convert(request.clone())
                })
                .map(|&c| c)
                .collect::<Vec<&'static dyn MessageConverter>>()
                .into_iter())
        }
    }

    pub trait Converters {
        fn converters(&self, request: &HttpRequest) -> Box<dyn Iterator<Item=&'static dyn MessageConverter>>;
    }

    impl ConverterContext for Context {
        fn convert<T: Serialize + for<'a> Deserialize<'a>>(&self, request: &HttpRequest) -> Option<MessageType<T>> {
            self.converters.converters(request)
                .find_map(|c| {
                    let found = (&c).convert(request.clone());
                    found
                })
        }
    }

    pub struct Context {
        pub converters: ConverterRegistry
    }

    impl Default for Context {
        fn default() -> Self {
            let mut registry = ConverterRegistry {
                converters: Box::new(LinkedList::new())
            };
            registry.register(&JsonMessageConverter {});
            Self {
                converters: registry
            }
        }
    }

    pub trait RequestExtractor<T> {
        fn convert_extract(&self, request: &HttpRequest) -> T;
    }

    impl RequestExtractor<EndpointMetadata> for Context {
        fn convert_extract(&self, request: &HttpRequest) -> EndpointMetadata {
            EndpointMetadata::default()
        }
    }

    pub trait ConverterContext {
        fn convert<T: Serialize + for<'a> Deserialize<'a>>(&self, request: &HttpRequest) -> Option<MessageType<T>>;
    }

    pub trait Controller<'a, T: Serialize, U: for<'b> Deserialize<'b>> {
        fn do_request(&self, request: HttpRequest, action: &'a dyn Fn(EndpointMetadata, U, &Context) -> T);
    }

    pub trait Filter {
        fn filter(&self, request: HttpRequest, response: HttpResponse, filter: FilterChain);
    }

    #[derive(Clone, Copy)]
    pub struct MessageType<T: Serialize> {
        pub message: T
    }

    impl <'a> MessageConverter for &'a dyn MessageConverter {
        fn do_convert(&self, request: HttpRequest) -> bool {
            (*self).do_convert(request)
        }
    }

    pub trait MessageConverter {
        fn convert<U: Serialize + for<'a> Deserialize<'a>>(&self, request: HttpRequest) -> Option<MessageType<U>> where Self: Sized {
            let option = JsonMessageConverter {}.convert(request);
            option
        }
        fn do_convert(&self, request: HttpRequest) -> bool;
    }

    pub struct JsonMessageConverter;

    impl MessageConverter for JsonMessageConverter {
        fn convert<U: Serialize + for<'a> Deserialize<'a>>(&self, request: HttpRequest) -> Option<MessageType<U>> {
            serde_json::from_str(&request.body)
                .ok()
                .map(|mr| {
                    let message_type: MessageType<U> = MessageType {
                        message: mr
                    };
                    message_type
                })
        }

        fn do_convert(&self, request: HttpRequest) -> bool {
            // request.headers["MediaType"].contains("json")
            true
        }
    }

}