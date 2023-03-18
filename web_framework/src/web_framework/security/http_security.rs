use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use web_framework_shared::request::WebRequest;
use crate::web_framework::context_builder::DelegatingAuthenticationManagerBuilder;
use crate::web_framework::convert::{Register, Registration};
use crate::web_framework::filter::filter::FilterChain;
use crate::web_framework::security::authorization::{AuthorityAuthorizationManager, AuthorizationManager, DelegatingAuthorizationManagerBuilder, RequestMatcherDelegatingAuthorizationManager, RequestMatcherEntry};
use crate::web_framework::security::authentication::{AuthenticationProvider, DelegatingAuthenticationManager};
use crate::web_framework::security::security_filter::SecurityFilterChain;

pub struct HttpSecurity<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub authentication_manager: Arc<Mutex<Option<DelegatingAuthenticationManagerBuilder>>>,
    pub authorization_manager: Arc<Mutex<Option<DelegatingAuthorizationManagerBuilder>>>,
    pub phantom_req: PhantomData<Request>,
    pub phantom_res: PhantomData<Response>
}

impl <Request, Response> Default for HttpSecurity<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    fn default() -> Self {
        Self {
            authentication_manager: Arc::new(Mutex::new(Some(DelegatingAuthenticationManagerBuilder::default()))),
            authorization_manager: Arc::new(Mutex::new(Some(DelegatingAuthorizationManagerBuilder::default()))),
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
                auth.authority_authorization_managers
                    .lock()
                    .map(|mut auth|
                        auth.push(RequestMatcherEntry::new(endpoints, authorities))
                    )
            });
        }).expect("Could not add request matcher to authorization manager.");
    }

    pub fn authentication_provider(&self, authentication_manager: Box<dyn AuthenticationProvider>) {
        self.authentication_manager.lock().map(|mut auth_manager| {
            auth_manager.as_mut().map(|auth_manager| {
                auth_manager.register(authentication_manager);
            })
        }).expect("Could not add delegating authentication manager.");
    }

    pub fn authorization_manager(&self, authentication_manager: RequestMatcherEntry<AuthorityAuthorizationManager<WebRequest>>) {
        self.authorization_manager.lock().map(|mut auth_manager| {
            auth_manager.as_mut().map(|auth_manager| {
                auth_manager.register(authentication_manager);
            })
        }).expect("Could not add delegating authentication manager.");
    }

    pub fn authorization_manager_builder(&mut self, mut authentication_manager: DelegatingAuthorizationManagerBuilder) {
        let _ = std::mem::replace(&mut self.authorization_manager.lock().unwrap().take(), Some(authentication_manager));
    }

    pub fn authentication_manager_builder(&mut self, mut authentication_manager: DelegatingAuthenticationManagerBuilder) {
        let _ = std::mem::replace(&mut self.authentication_manager.lock().unwrap().take(), Some(authentication_manager));
    }

}

