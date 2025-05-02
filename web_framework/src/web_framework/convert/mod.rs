
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
macro_rules! provide_default_message_converters {
    () => {
        const media_header_keys_c: &[&str] = &["MediaType", "mediatype", "Media-Type", "mediatype", "Content-Type", "ContentType", "content-type", "contenttype"];

        #[derive(Clone, Debug, Default)]
        pub struct JsonMessageConverter<Request, Response> {
            req: PhantomData<Request>,
            res: PhantomData<Response>
        }
        #[derive(Clone, Debug, Default)]
        pub struct HtmlMessageConverter<Request, Response> {
            req: PhantomData<Request>,
            res: PhantomData<Response>
        }

        pub fn media_header_keys() -> &'static [&'static str] {
            media_header_keys_c
        }

        fn retrieve_media_header<'a>(headers: &'a HashMap<String, String>) -> Option<&'a String> {
            for i in media_header_keys() {
                let header_key = &i.to_string();
                if headers.contains_key(header_key) {
                    return headers.get(header_key);
                }
            }

            None
        }


        impl<Request, Response> JsonMessageConverter<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
        {

            fn convert_json_to(
                &self,
                request: &WebRequest,
            ) -> Option<MessageType<Response>> {
                let result = serde_json::from_str(&request.body);
                match result {
                    Ok(mr) => {
                        let message_type: MessageType<Response> = MessageType { message: mr };
                        Some(message_type)
                    }
                    Err(err) => {
                        println!("Error {}!", err.to_string());
                        None
                    }
                }
            }

            fn do_convert_json(&self, request: &WebRequest) -> bool {
                for header in request.headers.iter() {
                    for potential_media in media_header_keys() {
                        if header.0 == potential_media && header.1.contains("json") {
                            return true;
                        }
                    }
                }
                false
            }

            fn convert_json_from(&self, request: &Response, web_request: &WebRequest) -> Option<String>
            where
                Self: Sized,
            {
                serde_json::to_string(&request).ok()
            }

        }


        impl<Request, Response> HtmlMessageConverter<Request, Response>
        where
            Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
            Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
        {
            fn convert_html_to(&self, request: &WebRequest) -> Option<MessageType<Request>> {
                todo!()
            }

            fn convert_html_from(&self,  request: &Response, request_body: &WebRequest) -> Option<String> {
                todo!()
            }

            fn do_convert_html(&self, request: &WebRequest) -> bool {
                for header in request.headers.iter() {
                    if (header.0 == "MediaType" || header.0 == "mediatype") && header.1.contains("json") {
                        return true;
                    }
                }
                false
            }


        }

    }
}

// example
// default_message_converters!();
// create_message_converter!((
//         (
//             ("application/json" as json => JsonMessageConverter<ReturnRequest, ReturnRequest>),
//             ("text/html" as html => HtmlMessageConverter<ReturnRequest, ReturnRequest>)
//         ) ===> (ReturnRequest => ReturnRequest),
//         (
//             ("application/json" as json => JsonMessageConverter<AnotherRequest, AnotherRequest>),
//             ("text/html" as html => HtmlMessageConverter<AnotherRequest, AnotherRequest>)
//         ) ===> (AnotherRequest => AnotherRequest)
//     )
//     => DelegatingMessageConverter);
#[macro_export]
macro_rules! create_delegating_message_converters {
    (($(($(($matcher:literal as $field_name:ident => $converter:ty)),*) ===> ($request:ty => $response:ty)),*) => $delegator:ident) => {
        paste::paste! {
            #[derive(Clone)]
            pub struct $delegator {
                $(
                    $(
                        [<$field_name:lower _ $request:lower>]: $converter,
                    )*
                )*
                media_types: Vec<String>
            }
        }

        impl $delegator {

            pub fn new() -> Self where Self: Sized {
                let mut media_types = vec![];
                $(
                    $(
                        let next_message_converter = <$converter as MessageConverter<$request,$response>>::new_message_converter();
                        let to_add = <$converter as MessageConverter<$request,$response>>::message_type(&next_message_converter);
                        for media_type in &to_add {
                            media_types.push(media_type.clone());
                        }
                    )*
                )*
                paste::paste!{
                    Self {
                            $(
                                $(
                                     [<$field_name:lower _ $request:lower>]: <$converter as MessageConverter<$request,$response>>::new_message_converter(),
                                )*
                            )*
                            media_types
                        }
                    }
                }
        }

        $(
            impl MessageConverter<$request, $response> for $delegator
            {

                fn new_message_converter() -> Self {
                    $delegator::new()
                }

                fn convert_to(
                    &self,
                    request: &WebRequest,
                ) -> Option<MessageType<$response>>
                where
                    Self: Sized,
                {
                    retrieve_media_header(&request.headers)
                        .and_then(|found| {
                            paste! {
                                $(
                                    if found == $matcher || found == $matcher {
                                        return <$converter as MessageConverter<$request, $response>>::convert_to(&self.[<$field_name:lower _ $request:lower>], request);
                                    }
                                )*
                            }

                            None
                        })
                }

                fn convert_from(&self, request_body: &$response, request: &WebRequest) -> Option<String>
                where
                    Self: Sized,
                {
                    retrieve_media_header(&request.headers)
                        .and_then(|found| {
                            paste! {
                                $(
                                    if found == $matcher || found == $matcher {
                                        return <$converter as MessageConverter<$request,$response>>::convert_from(&self.[<$field_name:lower _ $request:lower>], request_body, request);
                                    }
                                )*
                            }
                            None
                        })
                }

                fn do_convert(&self, request: &WebRequest) -> bool {
                    paste! {
                        $(
                            if <$converter as MessageConverter<$request,$response>>::do_convert(&self.[<$field_name:lower _ $request:lower>], request) {
                                return true;
                            }
                        )*
                        false
                    }
                }

                fn message_type(&self) -> Vec<String> {
                    self.media_types.clone()
                }
            }

            impl MessageConverter<$request, $response> for JsonMessageConverter<$request, $response>
            {

                fn new_message_converter() -> Self {
                    JsonMessageConverter::<$request, $response>::default()
                }

                fn convert_to(
                    &self,
                    request: &WebRequest,
                ) -> Option<MessageType<$request>> {
                    self.convert_json_to(request)
                }

                fn do_convert(&self, request: &WebRequest) -> bool {
                    self.do_convert_json(request)
                }

                fn convert_from(&self, request: &$response, web_request: &WebRequest) -> Option<String>
                where
                    Self: Sized {
                    self.convert_json_from(request, web_request)
                }

                fn message_type(&self) -> Vec<String> {
                    vec!["application/json".to_string()]
                }
            }


            impl MessageConverter<$request, $response> for HtmlMessageConverter<$request, $response>
            {
                fn new_message_converter() -> Self {
                    HtmlMessageConverter::<$request, $response>::default()
                }

                fn convert_to(&self, request: &WebRequest) -> Option<MessageType<$request>> {
                    self.convert_html_to(request)
                }

                fn convert_from(&self,  request: &$response, request_body: &WebRequest) -> Option<String> {
                    self.convert_html_from(request, request_body)
                }

                fn do_convert(&self, request: &WebRequest) -> bool {
                    self.do_convert_html(request)
                }


                fn message_type(&self) -> Vec<String> {
                    vec!["text/html".to_string()]
                }
            }
        )*

    }
}

pub trait MessageConverter<Request, Response>: Send + Sync
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{

    fn new_message_converter() -> Self where Self: Sized;

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
    fn new_message_converter() -> Self {
        DefaultMessageConverter::default()
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
