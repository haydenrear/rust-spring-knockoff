
use core::borrow::BorrowMut;
use crate::web_framework::context::RequestContext;
use crate::web_framework::filter::filter::MediaType;
use crate::web_framework::message::MessageType;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::LinkedList;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::vec;
use knockoff_security::knockoff_security::authentication_type::{AuthenticationConversionError, Unauthenticated};
use module_macro_lib::{AuthenticationType, AuthenticationTypeConverter};
use web_framework_shared::convert::Converter;
use crate::web_framework::security::security::{AuthenticationConverter, AuthenticationToken};
use web_framework_shared::request::{EndpointMetadata, WebRequest};
use knockoff_security::knockoff_security::authentication_type::AuthenticationAware;

#[macro_export]
macro_rules! default_message_converters {
    () => {
        #[derive(Clone)]
        pub struct JsonMessageConverterImpl;
        #[derive(Clone)]
        pub struct HtmlMessageConverter;
    }
}

#[macro_export]
macro_rules! create_message_converter {
    (($($converter_path:path => $converter_ident:expr =>> $matcher:literal => $converter:ty => $field_name:ident),*) ===> $gen:ty => $delegator:ident) => {

        use crate::*;
        $(
            use $converter_path;
        )*

        //TODO: have to edit this struct to add fields... adding the message converters in order to use
        // DelegatingMessageConverter in place of dyn MessageConverter so it won't be invoking generic
        // on trait - this
        #[derive(Clone)]
        pub struct $delegator{
            $(
                $field_name: $converter,
            )*
            media_types: Vec<String>
        }

        impl MessageConverter<$gen, $gen> for $delegator
        {

            fn new() -> Self where Self: Sized {
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

            fn convert_to(
                &self,
                request: &WebRequest,
            ) -> Option<MessageType<$gen>>
            where
                Self: Sized,
            {
                $(
                    if request.headers["MediaType"] == $matcher || request.headers["mediatype"] == $matcher {
                        return <$converter_path as MessageConverter<$gen, $gen>>::convert_to(&self.$field_name, request);
                    }
                )*
                None
            }

            fn convert_from(&self, request_body: &$gen, web_request: &WebRequest) -> Option<String>
            where
                Self: Sized,
            {
                $(
                    if web_request.headers["MediaType"] == $matcher || web_request.headers["mediatype"] == $matcher {
                        return <$converter_path as MessageConverter<$gen,$gen>>::convert_from(&self.$field_name, request_body, web_request);
                    }
                )*
                None
            }

            fn do_convert(&self, request: &WebRequest) -> bool {
                $(
                    if <$converter_path as MessageConverter<$gen,$gen>>::do_convert(&self.$field_name, request) {
                        return true;
                    }
                )*
                false
            }

            fn message_type(&self) -> Vec<String> {
                self.media_types.clone()
            }
        }

        impl MessageConverter<$gen, $gen> for JsonMessageConverterImpl
        {

            fn new() -> Self where Self: Sized {
                Self {}
            }

            fn convert_to(
                &self,
                request: &WebRequest,
            ) -> Option<MessageType<$gen>> {
                let result = serde_json::from_str(&request.body);
                match result {
                    Ok(mr) => {
                        let message_type: MessageType<$gen> = MessageType { message: mr };
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

            fn convert_from(&self, request: &$gen, web_request: &WebRequest) -> Option<String>
            where
                Self: Sized,
            {
                serde_json::to_string(&request).ok()
            }

            fn message_type(&self) -> Vec<String> {
                vec!["application/json".to_string()]
            }
        }


        impl MessageConverter<$gen, $gen> for HtmlMessageConverter
        {
            fn new() -> Self where Self: Sized {
                todo!()
            }

            fn convert_to(&self, request: &WebRequest) -> Option<MessageType<$gen>> {
                todo!()
            }

            fn convert_from(&self,  request: &$gen, request_body: &WebRequest) -> Option<String> {
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
                vec!["text/html".to_string()]
            }
        }

    }
}

pub trait MessageConverter<Request, Response>: Send + Sync
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{

    fn new() -> Self where Self: Sized;

    fn convert_to(
        &self,
        request: &WebRequest,
    ) -> Option<MessageType<Request>>;

    fn convert_from(&self, request_body: &Response, request: &WebRequest) -> Option<String>;

    fn do_convert(&self, request: &WebRequest) -> bool;

    fn message_type(&self) -> Vec<String>;

}


pub trait JsonMessageConverter<Request, Response>: MessageConverter<Request,Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
}



#[derive(Copy, Clone)]
pub struct OtherMessageConverter;

impl<Request, Response> MessageConverter<Request, Response> for OtherMessageConverter
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    fn new() -> Self where Self: Sized {
        todo!()
    }

    fn convert_to(
        &self,
        request: &WebRequest,
    ) -> Option<MessageType<Request>>
    where
        Self: Sized,
    {
        None
    }

    fn convert_from(&self, request: &Response, web_request: &WebRequest) -> Option<String>
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
pub struct ConverterRegistry<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub converters: Arc<Option<Box<dyn MessageConverter<Request, Response>>>>,
    pub request_convert: Arc<Option<Box<dyn RequestExtractor<EndpointMetadata>>>>
}

impl <Request, Response> ConverterRegistry<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub fn new(request_extractor: Option<Box<dyn RequestExtractor<EndpointMetadata>>>) -> ConverterRegistry<Request, Response> {
        Self {
            converters: Arc::new(None),
            request_convert: Arc::new(request_extractor),
        }
    }

}

pub struct EndpointRequestExtractor {
}

impl EndpointRequestExtractor {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl RequestExtractor<EndpointMetadata> for EndpointRequestExtractor  {
    fn convert_extract(&self, request: &WebRequest) -> Option<EndpointMetadata> {
        Some(EndpointMetadata::default())
    }
}

impl <Request, Response> ConverterRegistryContainer<Request, Response> for ConverterRegistry<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{

    fn converters(
        &self,
        request: &WebRequest,
    ) -> Arc<Option<Box<dyn MessageConverter<Request, Response>>>> {
        match self.converters.as_ref() {
            None => {
                Arc::new(None)
            }
            Some(converter) => {
                if converter.do_convert(request) {
                    return self.converters.clone()
                }
                Arc::new(None)
            }
        }
    }

    fn convert_from_converters(
        &self,
        media_type: String,
        response: &Response,
        request: &WebRequest
    ) -> Option<String> {
        match self.converters.as_ref() {
            None => {
                None
            }
            Some(converter) => {
                if converter.message_type()
                    .iter()
                    .any(|message_type| message_type.clone() == media_type) {
                    return self.converters.as_ref().as_ref()
                        .map(|c| c.convert_from(response, request))
                        .flatten()
                } else {
                    None
                }
            }
        }
    }
}

pub trait RequestExtractor<T>: Send + Sync {
    fn convert_extract(&self, request: &WebRequest) -> Option<T>;
}

impl <Request, Response> RequestExtractor<EndpointMetadata> for RequestContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    fn convert_extract(&self, request: &WebRequest) -> Option<EndpointMetadata> {
        match self.message_converters
            .request_convert
            .as_ref() {
            None => {
                None
            }
            Some(converter) => {
                converter.convert_extract(request)
            }
        }
    }
}

impl <Request, Response> Converters<Request, Response> for RequestContext<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    fn convert_to(
        &self,
        request: &WebRequest
    ) -> Option<MessageType<Request>> {
        self.message_converters.converters
            .as_ref()
            .as_ref()
            .filter(|converter| converter.do_convert(request))
            .map(|converter| converter.convert_to(request))
            .flatten()
    }

    fn convert_from(
        &self,
        request: &Response,
        web_request: &WebRequest,
        media_type: Option<String>
    ) -> Option<String> {
        self.message_converters.convert_from_converters(
            media_type.or(Some("application/json".to_string())).unwrap(),
            request,
            web_request
        )
    }
}


#[derive(Clone)]
pub struct AuthenticationConverterRegistry {
    pub converters: Arc<Vec<&'static dyn AuthenticationConverter>>,
    pub authentication_type_converter: Arc<&'static dyn AuthenticationTypeConverter>
}

impl Converter<WebRequest, Result<AuthenticationToken, AuthenticationConversionError>> for AuthenticationConverterRegistry {
    fn convert(&self, from: &WebRequest) -> Result<AuthenticationToken, AuthenticationConversionError> {
        self.authentication_type_converter.deref().convert(from)
            .map(|auth_type| auth_type.get_principal()
                .map(|principal| (auth_type, principal))
            )
            .map(|auth_type| {
                auth_type.map(|auth_type| {
                    AuthenticationToken {
                        name: auth_type.1,
                        auth: auth_type.0
                    }
                })
                    .or(Some(AuthenticationToken { name: "".to_string(), auth: AuthenticationType::Unauthenticated}))
                    .unwrap()
            })
            .or(Err(AuthenticationConversionError{ message: "Error processing authentication token.".to_string() }))
    }
}

pub trait Converters<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    fn convert_to(
        &self,
        request: &WebRequest,
    ) -> Option<MessageType<Request>>;
    fn convert_from(
        &self,
        request: &Response,
        web_request: &WebRequest,
        media_type: Option<String>,
    ) -> Option<String>;
}

pub trait  ConverterRegistryContainer<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    fn converters(
        &self,
        request: &WebRequest,
    ) -> Arc<Option<Box<dyn MessageConverter<Request, Response>>>>;
    fn convert_from_converters(
        &self,
        media_type: String,
        response: &Response,
        request: &WebRequest
    ) -> Option<String>;
}
