use std::collections::LinkedList;
use serde::{Deserialize, Serialize};
use crate::context::Context;
use crate::filter::filter::MediaType;
use crate::message::MessageType;
use crate::request::request::HttpRequest;

pub trait Converters {
    fn converters(&self, request: &HttpRequest) -> Box<dyn Iterator<Item=&'static dyn MessageConverter>>;
    fn convert_from_converters(&self, media_type: MediaType) -> Box<dyn Iterator<Item=&'static dyn MessageConverter>>;
}

impl <'a> MessageConverter for &'a dyn MessageConverter {
    fn do_convert(&self, request: HttpRequest) -> bool {
        (*self).do_convert(request)
    }

    fn message_type(&self) -> MediaType {
        MediaType::Json
    }
}

pub trait MessageConverter {
    fn convert_to<U: Serialize + for<'a> Deserialize<'a>>(
        &self,
        request: HttpRequest
    ) -> Option<MessageType<U>>
        where Self: Sized
    {
        let option = JsonMessageConverter {}.convert_to(request);
        option
    }

    fn convert_from<U: Serialize + for<'a> Deserialize<'a>>(
        &self,
        request: &U
    ) -> Option<String>
        where Self: Sized
    {
        let option = JsonMessageConverter {}.convert_from(request);
        option
    }
    fn do_convert(&self, request: HttpRequest) -> bool;
    fn message_type(&self) -> MediaType;
}

pub struct JsonMessageConverter;

impl MessageConverter for JsonMessageConverter {
    fn convert_to<U: Serialize + for<'a> Deserialize<'a>>(&self, request: HttpRequest) -> Option<MessageType<U>> {
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

    fn convert_from<U: Serialize + for<'a> Deserialize<'a>>(&self, request: &U) -> Option<String> where Self: Sized {
        serde_json::to_string(&request)
            .ok()
    }

    fn message_type(&self) -> MediaType {
        MediaType::Json
    }
}

pub struct OtherMessageConverter;

impl MessageConverter for OtherMessageConverter {
    fn convert_to<U: Serialize + for<'a> Deserialize<'a>>(&self, request: HttpRequest) -> Option<MessageType<U>> where Self: Sized {
        None
    }

    fn convert_from<U: Serialize + for<'a> Deserialize<'a>>(&self, request: &U) -> Option<String> where Self: Sized {
        None
    }

    fn do_convert(&self, request: HttpRequest) -> bool {
        false
    }

    fn message_type(&self) -> MediaType {
        MediaType::Json
    }
}

pub trait Registration<'a, C> {
    fn register(&mut self, converter: &'a C);
}

pub struct ConverterRegistry {
    pub converters: Box<LinkedList<&'static dyn MessageConverter>>
}

impl <'a> Registration<'a, JsonMessageConverter> for ConverterRegistry where 'a: 'static {
    fn register(&mut self, converter: &'a JsonMessageConverter) {
        self.converters.push_front(converter)
    }
}

impl <'a> Registration<'a, OtherMessageConverter> for ConverterRegistry where 'a: 'static {
    fn register(&mut self, converter: &'a OtherMessageConverter) {
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

    fn convert_from_converters(&self, media_type: MediaType) -> Box<dyn Iterator<Item=&'static dyn MessageConverter>> {
        Box::new(self.converters.iter()
            .filter(|&&c| c.message_type() == media_type)
            .map(|&c| c)
            .collect::<Vec<&'static dyn MessageConverter>>()
            .into_iter())
    }
}

impl ConverterContext for Context {
    fn convert_to<T: Serialize + for<'a> Deserialize<'a>>(&self, request: &HttpRequest) -> Option<MessageType<T>> {
        self.converters.converters(request)
            .find_map(|c| {
                let found = (&c).convert_to(request.clone());
                found
            })
    }

    fn convert_from<T: Serialize + for<'a> Deserialize<'a> + Clone>(&self, request: &T, media_type: MediaType) -> Option<String> {
        self.converters.convert_from_converters(media_type)
            .find_map(|c| {
                let found = (&c).convert_from(request);
                found
            })
    }
}


pub trait ConverterContext {
    fn convert_to<T: Serialize + for<'a> Deserialize<'a>>(&self, request: &HttpRequest) -> Option<MessageType<T>>;
    fn convert_from<T: Serialize + for<'a> Deserialize<'a> + Clone>(&self, request: &T, media_type: MediaType) -> Option<String>;
}
