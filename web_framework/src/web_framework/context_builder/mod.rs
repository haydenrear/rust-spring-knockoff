use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use knockoff_security::knockoff_security::authentication_type::{AuthenticationAware, AuthenticationConversionError};
use module_macro_lib::{AuthenticationTypeConverter, AuthenticationTypeConverterImpl};
use web_framework_shared::convert::Converter;
use crate::web_framework::context::{ApplicationContext, FilterRegistrar, RequestContext};
use web_framework_shared::request::{EndpointMetadata, WebRequest};
use crate::web_framework::convert::{AuthenticationConverterRegistry, ConverterRegistry, DefaultMessageConverter, EndpointRequestExtractor, MessageConverter, OtherMessageConverter, Registration, RequestExtractor};
use crate::web_framework::security::authentication::{AuthenticationConverter, AuthenticationProvider, AuthenticationToken, DelegatingAuthenticationManager};

/// TODO: This should be ControllerEndpointBuilder, and the user should provide
///  a module annotated with controller or rest_controller to build each one.
///  -- then, the application context will be built with all of them.
impl<Request, Response> ApplicationContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    fn new() -> ApplicationContextBuilder<Request, Response> {
        Self {
            filter_registry: Some(Arc::new(Mutex::new(FilterRegistrar::new()))),
            request_context_builder: Some(Arc::new(Mutex::new(RequestContextBuilder::new()))),
            authentication_converters: None,
        }
    }
}

pub struct ApplicationContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub filter_registry: Option<Arc<Mutex<FilterRegistrar<Request, Response>>>>,
    pub request_context_builder: Option<Arc<Mutex<RequestContextBuilder<Request, Response>>>>,
    pub authentication_converters: Option<Arc<AuthenticationConverterRegistryBuilder>>,
}

impl<Request, Response> ApplicationContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub fn build(&self) -> ApplicationContext<Request, Response> {
        let mut filter_registry_found = self.filter_registry
            .as_ref().unwrap().lock().unwrap().clone();
        filter_registry_found.build();
        let context = self.request_context_builder.as_ref()
            .unwrap().lock().unwrap().build();
        ApplicationContext {
            filter_registry: Arc::new(filter_registry_found),
            request_context: context,
            authentication_converters: self.authentication_converters.as_ref().unwrap().build(),
        }
    }
}

impl<Request, Response> Registration<dyn AuthenticationConverter> for ApplicationContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    fn register(&self, converter: Box<dyn AuthenticationConverter>) {
        self.authentication_converters.as_ref()
            .map(|r| {
                r.register(converter)
            });
    }
}

#[derive(Clone)]
pub struct ConverterRegistryBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub converters: Arc<Mutex<Option<Box<dyn MessageConverter<Request, Response>>>>>,
    pub request_convert: Arc<Mutex<Option<Box<dyn RequestExtractor<EndpointMetadata>>>>>,
}

impl<Request, Response> ConverterRegistryBuilder<Request, Response> where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    fn new() -> Self {
        Self {
            converters: Arc::new(Mutex::new(Some(Box::new(DefaultMessageConverter::default())))),
            request_convert: Arc::new(Mutex::new(Some(Box::new(EndpointRequestExtractor::default())))),
        }
    }
}

impl<Request, Response> ConverterRegistryBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub fn build(&mut self) -> ConverterRegistry<Request, Response> {
        let mut to_switch: Option<Box<dyn MessageConverter<Request, Response>>> = None;
        std::mem::swap(&mut to_switch, &mut self.converters.lock().unwrap().take());
        let mut request_extractor_found: Option<Box<dyn RequestExtractor<EndpointMetadata>>> = None;
        std::mem::swap(&mut request_extractor_found, &mut self.request_convert.lock().unwrap().take());
        ConverterRegistry {
            converters: Arc::new(to_switch.unwrap()),
            request_convert: Arc::new(request_extractor_found.unwrap()),
        }
    }
}

#[derive(Clone)]
pub struct RequestContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub message_converter_builder: ConverterRegistryBuilder<Request, Response>,
    pub authentication_manager_builder: DelegatingAuthenticationManagerBuilder,
}

impl<Request, Response> RequestContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            message_converter_builder: ConverterRegistryBuilder::new(),
            authentication_manager_builder: DelegatingAuthenticationManagerBuilder::new(),
        }
    }
}

impl<Request, Response> RequestContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    fn build(&mut self) -> RequestContext<Request, Response> {
        RequestContext {
            message_converters: self.message_converter_builder.build(),
            authentication_manager: self.authentication_manager_builder.build(),
        }
    }
}


impl<Request, Response> Default for RequestContextBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    fn default() -> Self {
        let mut registry = ConverterRegistryBuilder {
            converters: Arc::new(Mutex::new(Some(Box::new(OtherMessageConverter {})))),
            request_convert: Arc::new(Mutex::new(Some(Box::new(EndpointRequestExtractor {})))),
        };
        Self {
            message_converter_builder: registry,
            authentication_manager_builder: DelegatingAuthenticationManagerBuilder {
                providers: Arc::new(Mutex::new(vec![].into()))
            },
        }
    }
}

#[deny(Clone)]
pub struct DispatcherBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    pub context: RequestContextBuilder<Request, Response>,
}

#[derive(Clone)]
pub struct DelegatingAuthenticationManagerBuilder {
    pub providers: Arc<Mutex<Vec<Box<dyn AuthenticationProvider>>>>,
}

impl DelegatingAuthenticationManagerBuilder {
    pub fn new() -> Self {
        DelegatingAuthenticationManagerBuilder {
            providers: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn add_provider(&self, auth: Box<dyn AuthenticationProvider>) {
        self.providers.as_ref().lock().unwrap().push(auth)
    }

    pub fn get_provider(&self) -> Vec<Box<dyn AuthenticationProvider>> {
        let mut guard = self.providers.as_ref().lock().unwrap();
        let mut next = vec![];
        std::mem::swap(&mut next, guard.as_mut());
        next
    }
}

impl DelegatingAuthenticationManagerBuilder {
    pub(crate) fn build(&self) -> DelegatingAuthenticationManager {
        DelegatingAuthenticationManager {
            providers: Arc::new(self.get_provider()),
        }
    }
}

#[derive(Clone)]
pub struct AuthenticationConverterRegistryBuilder {
    pub converters: Arc<Mutex<Vec<Box<dyn AuthenticationConverter>>>>,
    pub authentication_type_converter: Arc<Mutex<Option<Box<dyn AuthenticationTypeConverter>>>>,
}

impl AuthenticationConverterRegistryBuilder {
    pub(crate) fn build(&self) -> AuthenticationConverterRegistry {
        AuthenticationConverterRegistry {
            converters: Arc::new(self.get_converter()),
            authentication_type_converter: Arc::new(self.get_type_converter().unwrap()),
        }
    }

    pub(crate) fn new() -> AuthenticationConverterRegistryBuilder {
        Self {
            converters: Arc::new(Mutex::new(vec![])),
            authentication_type_converter: Arc::new(Mutex::new(Some(Box::new(AuthenticationTypeConverterImpl::new())))),
        }
    }

    pub(crate) fn add_converter(&self, auth_converter: Box<dyn AuthenticationConverter>) {
        self.converters.as_ref().lock().unwrap().push(auth_converter)
    }

    pub(crate) fn add_type_converter(&self, auth_type_converter: Box<dyn AuthenticationTypeConverter>) {
        *self.authentication_type_converter.lock().unwrap() = Some(auth_type_converter);
    }

    pub fn get_converter(&self) -> Vec<Box<dyn AuthenticationConverter>> {
        let mut guard = self.converters.as_ref().lock().unwrap();
        let mut next = vec![];
        std::mem::swap(&mut next, guard.as_mut());
        next
    }

    pub fn get_type_converter(&self) -> Option<Box<dyn AuthenticationTypeConverter>> {
        let mut type_converter = self.authentication_type_converter.as_ref().lock().unwrap();
        let mut next = None;
        std::mem::swap(&mut next, &mut type_converter.take());
        next
    }

}

impl Registration<dyn AuthenticationConverter> for AuthenticationConverterRegistryBuilder
{
    fn register(&self, converter: Box<dyn AuthenticationConverter>) {
        self.converters.lock().unwrap().push(converter);
    }
}

impl AuthenticationConverterRegistry {
    pub fn new() -> Self {
        Self {
            converters: Arc::new(vec![]),
            authentication_type_converter: Arc::new(Box::new(AuthenticationTypeConverterImpl { })),
        }
    }
}
