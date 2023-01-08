use core::borrow::BorrowMut;
use crate::web_framework::context::RequestContext;
use crate::web_framework::filter::filter::MediaType;
use crate::web_framework::message::MessageType;
use crate::web_framework::request::request::{EndpointMetadata, WebRequest};
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::LinkedList;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::vec;

impl<'a> MessageConverter for &'a dyn MessageConverter {
    fn do_convert(&self, request: &WebRequest) -> bool {
        (*self).do_convert(request)
    }

    fn message_type(&self) -> Vec<String> {
        vec!["application/json".to_string()]
    }
}

#[macro_export]
macro_rules! create_message_converter {
    ($(($converter_path:path => $converter_ident:expr =>> $matcher:literal => $converter:ty => $field_name:ident)),*) => {

        use web_framework::web_framework::convert::MessageConverter;
        use web_framework::web_framework::message::MessageType;
        use web_framework::web_framework::filter::filter::MediaType;
        $(
            use $converter_path::*;
        )*

        #[derive(Clone)]
        pub struct DelegatingMessageConverter {
            $(
                $field_name: $converter,
            )*
            media_types: Vec<String>
        }

        impl DelegatingMessageConverter where Self: 'static {
            fn new() -> Self {
                let mut media_types = vec![];
                $(
                    let to_add = $converter_ident.message_type();
                    for media_type in &to_add {
                        media_types.push(media_type.clone());
                    }
                )*
                Self {
                    $(
                        $field_name: $converter_ident,
                    )*
                    media_types
                }
            }
        }

        impl MessageConverter for DelegatingMessageConverter {

            fn convert_to<U: Serialize + for<'a> Deserialize<'a>>(
                &self,
                request: &WebRequest,
            ) -> Option<MessageType<U>>
            where
                Self: Sized,
            {
                $(
                    if request.headers["MediaType"] == $matcher || request.headers["mediatype"] == $matcher {
                        return self.$field_name.convert_to(request);
                    }
                )*
                None
            }

            fn convert_from<U: Serialize + for<'a> Deserialize<'a>>(&self, request_body: &U, web_request: &WebRequest) -> Option<String>
            where
                Self: Sized,
            {
                $(
                    if web_request.headers["MediaType"] == $matcher || web_request.headers["mediatype"] == $matcher {
                        return self.$field_name.convert_from(request_body, web_request);
                    }
                )*
                None
            }

            fn do_convert(&self, request: &WebRequest) -> bool {
                $(
                    if self.$field_name.do_convert(request) {
                        return true;
                    }
                )*
                false
            }

            fn message_type(&self) -> Vec<String> {
                self.media_types.clone()
            }
        }
    }
}

pub struct MessageConverterBuilder {
    builders: Vec<(Box<dyn MessageConverter>, String)>
}

impl MessageConverterBuilder {
    pub fn add(&mut self, tuple: (Box<dyn MessageConverter>, String)) {
        self.builders.push(tuple)
    }
}

pub trait MessageConverter: Send + Sync {
    fn convert_to<U: Serialize + for<'a> Deserialize<'a>>(
        &self,
        request: &WebRequest,
    ) -> Option<MessageType<U>>
    where
        Self: Sized,
    {
        let option = JsonMessageConverter {}.convert_to(request);
        option
    }

    fn convert_from<U: Serialize + for<'a> Deserialize<'a>>(&self, request_body: &U, request: &WebRequest,) -> Option<String>
    where
        Self: Sized,
    {
        let option = JsonMessageConverter {}.convert_from(request_body, request);
        option
    }

    fn do_convert(&self, request: &WebRequest) -> bool;

    fn message_type(&self) -> Vec<String>;

}

#[derive(Clone)]
pub struct HtmlMessageConverter;

impl MessageConverter for HtmlMessageConverter {

    fn convert_to<U: Serialize + for<'a> Deserialize<'a>>(&self, request: &WebRequest) -> Option<MessageType<U>> where Self: Sized {
        todo!()
    }

    fn convert_from<U: Serialize + for<'a> Deserialize<'a>>(&self,  request: &U, request_body: &WebRequest) -> Option<String> where Self: Sized {
        todo!()
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        for header in request.headers.iter() {
            if (header.0 == "MediaType" || header.0 == "mediatype") && header.1.contains("json") {
                return true;
            }
        }
        false
    }


    fn message_type(&self) -> Vec<String> {
        vec!["application/json".to_string()]
    }
}

#[derive(Clone)]
pub struct JsonMessageConverter;

impl MessageConverter for JsonMessageConverter {

    fn convert_to<U: Serialize + for<'a> Deserialize<'a>>(
        &self,
        request: &WebRequest,
    ) -> Option<MessageType<U>> {
        let result = serde_json::from_str(&request.body);
        match result {
            Ok(mr) => {
                let message_type: MessageType<U> = MessageType { message: mr };
                Some(message_type)
            }
            Err(err) => {
                println!("Error {}!", err.to_string());
                None
            }
        }
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        for header in request.headers.iter() {
            if (header.0 == "MediaType" || header.0 == "mediatype") && header.1.contains("json") {
                return true;
            }
        }
        false
    }

    fn convert_from<U: Serialize + for<'a> Deserialize<'a>>(&self, request: &U, web_request: &WebRequest) -> Option<String>
    where
        Self: Sized,
    {
        serde_json::to_string(&request).ok()
    }

    fn message_type(&self) -> Vec<String> {
        vec!["application/json".to_string()]
    }
}

#[derive(Copy, Clone)]
pub struct OtherMessageConverter;

impl MessageConverter for OtherMessageConverter {
    fn convert_to<U: Serialize + for<'a> Deserialize<'a>>(
        &self,
        request: &WebRequest,
    ) -> Option<MessageType<U>>
    where
        Self: Sized,
    {
        None
    }

    fn convert_from<U: Serialize + for<'a> Deserialize<'a>>(&self, request: &U, web_request: &WebRequest) -> Option<String>
    where
        Self: Sized,
    {
        None
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        false
    }

    fn message_type(&self) -> Vec<String> {
        vec!["application/json".to_string()]
    }
}

pub trait Registration<'a, C: ?Sized> {
    fn register(&self, converter: &'a C);
}

pub trait Registry<C: ?Sized> {
    fn read_only_registrations(&self) -> Box<LinkedList<&'static C>>;
}

#[derive(Clone, Default)]
pub struct ConverterRegistry {
    pub converters: Arc<Option<Box<dyn MessageConverter>>>,
    pub request_convert: Arc<Option<&'static dyn RequestExtractor<EndpointMetadata>>>
}


#[derive(Clone)]
pub struct ConverterRegistryBuilder {
    pub converters: Arc<Mutex<Option<Box<dyn MessageConverter>>>>,
    pub request_convert: Arc<Mutex<Option<&'static dyn RequestExtractor<EndpointMetadata>>>>
}

impl ConverterRegistryBuilder {
    pub fn build(&mut self) -> ConverterRegistry {
        let mut to_switch: Option<Box<dyn MessageConverter>> = None;
        std::mem::swap(&mut to_switch, &mut self.converters.lock().unwrap().take());
        ConverterRegistry {
            converters: Arc::new(to_switch),
            request_convert: Arc::new(self.request_convert.lock().unwrap().clone())
        }
    }
}

impl ConverterRegistry {

    pub fn new(request_extractor: Option<&'static dyn RequestExtractor<EndpointMetadata>>) -> ConverterRegistry {
        Self {
            converters: Arc::new(None),
            request_convert: Arc::new(request_extractor),
        }
    }
}

pub struct EndpointRequestExtractor {
}

impl RequestExtractor<EndpointMetadata> for EndpointRequestExtractor  {
    fn convert_extract(&self, request: &WebRequest) -> Option<EndpointMetadata> {
        Some(EndpointMetadata::default())
    }
}

impl<'a> Registration<'a, dyn MessageConverter> for ConverterRegistryBuilder
where
    'a: 'static,
{
    fn register(&self, converter: &'a dyn MessageConverter) {
        // self.converters.lock().unwrap().borrow_mut().push(converter.clone())
    }
}


impl ConverterRegistryContainer for ConverterRegistry {

    fn converters(
        &self,
        request: &WebRequest,
    ) -> Box<dyn Iterator<Item = &'static dyn MessageConverter>> {
        Box::new(
            // self.converters
            //     .iter()
            //     .filter(|&c| c.do_convert(request))
            //     .map(|&c| c)
            //     .collect::<Vec<&'static dyn MessageConverter>>()
            //     .into_iter(),
            vec![].into_iter()
        )
    }

    fn convert_from_converters(
        &self,
        media_type: String,
    ) -> Box<dyn Iterator<Item = &'static dyn MessageConverter>> {
        Box::new(
            // self.converters
            //     .iter()
            //     .filter(|&&c| c.message_type()
            //         .iter()
            //         .any(|f| f.clone() == media_type)
            //     )
            //     .map(|&c| c)
            //     .collect::<Vec<&'static dyn MessageConverter>>()
            //     .into_iter(),
            vec![].into_iter()
        )
    }
}

pub trait RequestExtractor<T>: Send + Sync {
    fn convert_extract(&self, request: &WebRequest) -> Option<T>;
}

impl RequestExtractor<EndpointMetadata> for RequestContext {
    fn convert_extract(&self, request: &WebRequest) -> Option<EndpointMetadata> {
        self.message_converters
            .request_convert
            .map(|converter| converter.convert_extract(request).or(None))
            .unwrap()
    }
}

impl Converters for RequestContext {
    fn convert_to<T: Serialize + for<'a> Deserialize<'a>>(
        &self,
        request: &WebRequest,
    ) -> Option<MessageType<T>> {
        self.message_converters.converters(request).find_map(|c| {
            let found = (&c).convert_to(request);
            // let found = (&c).convert_to(request);
            found
        })
    }

    fn convert_from<T: Serialize + for<'a> Deserialize<'a> + Clone>(
        &self,
        request: &T,
        web_request: &WebRequest,
        media_type: Option<String>
    ) -> Option<String> {
        self.message_converters
            .convert_from_converters(media_type
                .or(Some("application/json".to_string())).unwrap()
            )
            .find_map(|c| {
                let found = (&c).convert_from(request, web_request);
                found
            })
    }
}

pub trait Converters {
    fn convert_to<T: Serialize + for<'a> Deserialize<'a>>(
        &self,
        request: &WebRequest,
    ) -> Option<MessageType<T>>;
    fn convert_from<T: Serialize + for<'a> Deserialize<'a> + Clone>(
        &self,
        request: &T,
        web_request: &WebRequest,
        media_type: Option<String>,
    ) -> Option<String>;
}

pub trait ConverterRegistryContainer {
    fn converters(
        &self,
        request: &WebRequest,
    ) -> Box<dyn Iterator<Item = &'static dyn MessageConverter>> ;
    fn convert_from_converters(
        &self,
        media_type: String,
    ) -> Box<dyn Iterator<Item = &'static dyn MessageConverter>> ;
}
