use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::web_framework::filter::filter::FilterChain;
use crate::web_framework::security::authorization::{AuthorizationManager, RequestMatcherDelegatingAuthorizationManager, RequestMatcherEntry};
use crate::web_framework::security::authentication::DelegatingAuthenticationManager;
use crate::web_framework::security::security_filter::SecurityFilterChain;

pub struct HttpSecurity<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub authentication_manager: Arc<Mutex<Option<DelegatingAuthenticationManager>>>,
    pub authorization_manager: Arc<Mutex<Option<RequestMatcherDelegatingAuthorizationManager>>>,
    pub phantom_req: PhantomData<Request>,
    pub phantom_res: PhantomData<Response>
}

impl <Request, Response> Default for HttpSecurity<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    fn default() -> Self {
        Self {
            authentication_manager: Arc::new(Mutex::new(None)),
            authorization_manager: Arc::new(Mutex::new(Some(RequestMatcherDelegatingAuthorizationManager::default()))),
            phantom_req: Default::default(),
            phantom_res: Default::default(),
        }
    }
}

impl <Request, Response> HttpSecurity<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub fn perform_build() -> SecurityFilterChain<Request, Response> {
        // TODO: add authorization filter and authentication filter.
        SecurityFilterChain {
            filters: Arc::new(FilterChain::default())
        }
    }

    pub fn http() -> HttpSecurity<Request, Response> {
        HttpSecurity::<Request, Response>::default()
    }

    pub fn request_matcher(&self, endpoints: Vec<&str>, authorities: Vec<&str>) {
        self.authorization_manager.lock().map(|mut auth| {
            auth.as_mut().map(|mut auth| {
                auth.authority_authorization_managers.push(RequestMatcherEntry::new(endpoints, authorities))
            });
        }).expect("Could not add request matcher to authorization manager.");
    }

    pub fn authorization_manager(&self, authorization_manager: DelegatingAuthenticationManager) {
        *self.authentication_manager.lock().unwrap() = Some(authorization_manager);
    }
}

