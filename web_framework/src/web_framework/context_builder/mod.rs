use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use core::borrow::BorrowMut;
use knockoff_security::knockoff_security::authentication_type::{AuthenticationAware, AuthenticationConversionError};
use module_macro_lib::{AuthenticationTypeConverter, AuthenticationTypeConverterImpl};
use web_framework_shared::convert::Converter;
use crate::web_framework::context::{ApplicationContext, RequestContext};
use web_framework_shared::request::{EndpointMetadata, WebRequest};
use crate::web_framework::convert::{AuthenticationConverterRegistry, ConverterRegistry, DefaultMessageConverter, EndpointRequestExtractor, MessageConverter, Registration, RequestExtractor};
use crate::web_framework::filter::filter::{DelegatingFilterProxy, Filter};
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
            filter_registry: Some(Arc::new(Mutex::new(FilterRegistrarBuilder::new()))),
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
    pub filter_registry: Option<Arc<Mutex<FilterRegistrarBuilder<Request, Response>>>>,
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
            .map(|r| r.register(converter));
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
            converters: Arc::new(Mutex::new(Some(Box::new(DefaultMessageConverter {})))),
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

    pub fn get_provider(&self) -> Vec<Box<dyn AuthenticationProvider>> {
        let mut guard = self.providers.as_ref().lock().unwrap();
        let mut next = vec![];
        std::mem::swap(&mut next, guard.as_mut());
        next
    }

}

impl Registration<dyn AuthenticationProvider> for DelegatingAuthenticationManagerBuilder {
    fn register(&self, auth: Box<dyn AuthenticationProvider>) {
        self.providers.as_ref().lock().unwrap().push(auth)
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

impl Registration<dyn AuthenticationConverter> for AuthenticationConverterRegistryBuilder {
    fn register(&self, converter: Box<dyn AuthenticationConverter>) {
        self.converters.lock().unwrap().push(converter);
    }
}

impl Registration<dyn AuthenticationTypeConverter> for AuthenticationConverterRegistryBuilder {
    fn register(&self, converter: Box<dyn AuthenticationTypeConverter>) {
        *self.authentication_type_converter.lock().unwrap() = Some(converter);
    }
}

impl AuthenticationConverterRegistryBuilder {
    pub fn register_authentication_converter<T: AuthenticationConverter + 'static>(&self, item: T) {
        self.register(Box::new(item) as Box<dyn AuthenticationConverter>)
    }

    pub fn register_authentication_type_converter<T: AuthenticationTypeConverter + 'static>(&self, item: T) {
        self.register(Box::new(item) as Box<dyn AuthenticationTypeConverter>)
    }

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

impl AuthenticationConverterRegistry {
    pub fn new() -> Self {
        Self {
            converters: Arc::new(vec![]),
            authentication_type_converter: Arc::new(Box::new(AuthenticationTypeConverterImpl::new())),
        }
    }
}

impl <Request, Response> FilterRegistrarBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub fn register(&mut self, converter: Filter<Request, Response>) {
        self.filters.lock().unwrap().borrow_mut().push(converter)
    }
}

pub struct FilterRegistrarBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
{
    pub filters: Arc<Mutex<Vec<Filter<Request, Response>>>>,
    pub fiter_chain: Arc<DelegatingFilterProxy<Request, Response>>,
    pub already_built: bool,
}

impl <Request, Response> FilterRegistrarBuilder<Request, Response>
where
    Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
    Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{

    // Sets the filter_build for later - so you don't have to do it every time.
    pub fn build(&mut self) -> Arc<DelegatingFilterProxy<Request, Response>> {
        self.do_build_inner();
        self.fiter_chain.clone()
    }

    fn do_build_inner(&mut self) {
        if !self.already_built {
            self.already_built = true;
            let mut filters_found = self.get_filters();
            filters_found.sort();
            self.fiter_chain = Arc::new(DelegatingFilterProxy::new(filters_found));
        }
    }
}

impl <Request, Response> Clone for FilterRegistrarBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync

{
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone(),
            fiter_chain: self.fiter_chain.clone(),
            already_built: self.already_built
        }
    }
}

impl <Request, Response> FilterRegistrarBuilder<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync
{
    pub(crate) fn new() -> FilterRegistrarBuilder<Request, Response> {
        Self {
            filters: Arc::new(Mutex::new(vec![])),
            fiter_chain: Arc::new(DelegatingFilterProxy::default()),
            already_built: false
        }
    }

    pub fn get_filters(&self) -> Vec<Filter<Request, Response>> {
        let mut guard = self.filters.as_ref().lock().unwrap();
        let mut next = vec![];
        for f in guard.iter() {
            next.push(f.to_owned())
        }
        next
    }

}
