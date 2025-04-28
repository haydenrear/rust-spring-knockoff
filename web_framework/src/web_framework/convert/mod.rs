
use core::borrow::BorrowMut;
use crate::web_framework::context::RequestHelpers;
use crate::web_framework::filter::filter::MediaType;
use crate::web_framework::message::MessageType;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::LinkedList;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::vec;
use knockoff_security::knockoff_security::authentication_type::{AuthenticationConversionError, Anonymous};
use authentication_gen::{AuthenticationType, AuthenticationTypeConverter};
use web_framework_shared::convert::Converter;
use web_framework_shared::request::{EndpointMetadata, WebRequest};
use knockoff_security::knockoff_security::authentication_type::AuthenticationAware;
use crate::web_framework::security::authentication::{Authentication, AuthenticationConverter, AuthenticationDetails, AuthenticationToken};

#[macro_export]
macro_rules! default_message_converters {
    () => {
        #[derive(Clone, Debug, Default)]
        pub struct JsonMessageConverter;
        #[derive(Clone, Debug, Default)]
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

        #[derive(Clone)]
        pub struct $delegator{
            $(
                $field_name: $converter,
            )*
            media_types: Vec<String>
        }

        impl $delegator {

            fn new() -> Self {
                let mut media = vec![];

                $(
                    media.push(String::from($matcher));
                )*

                Self {
                    media_types: media,
                    $(
                        $field_name: $converter_ident,
                    )*
                }
            }
        }

        fn retrieve_media_header<'a>(headers: &'a HashMap<String, String>) -> Option<&'a String> {
                headers.get("MediaType").or_else(|| headers.get("mediatype"))
                    .or_else(|| headers.get("Media-Type"))
                    .or_else(|| headers.get("mediatype"))
                    .or_else(|| headers.get("Content-Type"))
                    .or_else(|| headers.get("ContentType"))
                    .or_else(|| headers.get("content-type"))
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
                retrieve_media_header(&request.headers)
                    .and_then(|found| {
                        $(
                            if found == $matcher || found == $matcher {
                                return <$converter_path as MessageConverter<$gen, $gen>>::convert_to(&self.$field_name, request);
                            }
                        )*

                        None
                    })
            }

            fn convert_from(&self, request_body: &$gen, request: &WebRequest) -> Option<String>
            where
                Self: Sized,
            {
                retrieve_media_header(&request.headers)
                    .and_then(|found| {
                        $(
                            if found == $matcher || found == $matcher {
                                return <$converter_path as MessageConverter<$gen,$gen>>::convert_from(&self.$field_name, request_body, request);
                            }
                        )*
                        None
                    })
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

        impl MessageConverter<$gen, $gen> for JsonMessageConverter
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


#[derive(Default)]
pub struct DefaultMessageConverter;

impl<Request, Response> MessageConverter<Request, Response> for DefaultMessageConverter
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
        Self: Sized,{
        None
    }

    fn convert_from(&self, request: &Response, web_request: &WebRequest) -> Option<String>
    where
        Self: Sized,{
        serde_json::to_string(request).ok()
    }

    fn do_convert(&self, request: &WebRequest) -> bool {
        false
    }

    fn message_type(&self) -> Vec<String> {
        vec!["application/json".to_string()]
    }
}

pub trait Registration<C: ?Sized> {
    fn register(&self, converter: Box<C>);
}

pub trait Register<C> {
    fn register(&self, converter: C);
}

pub trait Registry<C: ?Sized> {
    fn read_only_registrations(&self) -> Arc<Vec<Box<C>>>;
}

#[derive(Clone)]
pub struct ConverterRegistry<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub converters: Arc<Box<dyn MessageConverter<Request, Response>>>,
    pub request_convert: Arc<Box<dyn RequestTypeExtractor<WebRequest, EndpointMetadata>>>
}

impl <Request, Response> Default for ConverterRegistry<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    fn default() -> Self {
        Self {
            converters: Arc::new(Box::new(DefaultMessageConverter::default())),
            request_convert: Arc::new(Box::new(EndpointRequestExtractor::default()))
        }
    }
}

impl <Request, Response> ConverterRegistry<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{

    pub fn new(request_extractor: Option<Box<dyn RequestTypeExtractor<WebRequest, EndpointMetadata>>>,
               message_converter: Option<Box<dyn MessageConverter<Request, Response>>>
    ) -> ConverterRegistry<Request, Response> {
        let request_convert
            = Arc::new(request_extractor.unwrap_or(Box::new(EndpointRequestExtractor::new())));
        let converters
            = Arc::new(message_converter.unwrap_or(Box::new(DefaultMessageConverter::default())));
        Self {
            converters,
            request_convert
        }
    }

}

#[derive(Default)]
pub struct EndpointRequestExtractor {
}

impl EndpointRequestExtractor {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl RequestTypeExtractor<WebRequest, EndpointMetadata> for EndpointRequestExtractor  {
    fn convert_extract(&self, request: &WebRequest) -> Option<EndpointMetadata> {
        // TODO:
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
    ) -> Option<Arc<Box<dyn MessageConverter<Request, Response>>>>{
        if self.converters.do_convert(request) {
            return Some(self.converters.clone());
        }
        None
    }

    fn convert_from_converters(
        &self,
        media_type: String,
        response: &Response,
        request: &WebRequest
    ) -> Option<String> {
        if self.converters.message_type()
            .iter()
            .any(|message_type| message_type.clone() == media_type) {
            return self.converters.convert_from(response, request);
        } else {
            None
        }
    }
}

pub trait RequestTypeExtractor<RequestT, T>: Send + Sync
    where
        RequestT: Default + Send + Sync + 'static{
    fn convert_extract(&self, request: &RequestT) -> Option<T>;
}

impl <Request, Response> RequestTypeExtractor<WebRequest, EndpointMetadata> for RequestHelpers<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    fn convert_extract(&self, request: &WebRequest) -> Option<EndpointMetadata> {
        self.message_converters.request_convert.convert_extract(request)
    }
}

impl <Request, Response> Converters<Request, Response> for RequestHelpers<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    fn convert_to(
        &self,
        request: &WebRequest
    ) -> Option<MessageType<Request>> {
        if self.message_converters.converters.do_convert(request) {
            return self.message_converters.converters.convert_to(request);
        }
        None
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
    pub converters: Arc<Vec<Box<dyn AuthenticationConverter>>>,
    pub authentication_type_converter: Arc<Box<dyn AuthenticationTypeConverter>>
}

impl Converter<WebRequest, Result<Authentication, AuthenticationConversionError>> for AuthenticationConverterRegistry {
    fn convert(&self, from: &WebRequest) -> Result<Authentication, AuthenticationConversionError> {
        self.authentication_type_converter.deref().convert(from)
            .map(|auth_type| {
                let authorities = auth_type.get_authorities().clone();
                auth_type.get_principal().map(|principal| {
                    AuthenticationToken {
                        name: principal,
                        auth: auth_type,
                        authenticated: false,
                        authorities,
                    }
                })
                .map(|auth_token| {
                    self.convert(&(&auth_token, from))
                })
                .or(Some(Err(AuthenticationConversionError{ message: "Error processing authentication token.".to_string() })))
                .unwrap()
            })
            .or(Err(AuthenticationConversionError{ message: Self::error_processing_authentication_message() }))
            .unwrap()
    }
}

impl AuthenticationConverterRegistry {
    fn error_processing_authentication_message() -> String {
        "Error processing authentication token.".to_string()
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
    ) -> Option<Arc<Box<dyn MessageConverter<Request, Response>>>>;
    fn convert_from_converters(
        &self,
        media_type: String,
        response: &Response,
        request: &WebRequest
    ) -> Option<String>;
}
