use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::web_framework::filter::filter::DelegatingFilterProxy;
use crate::web_framework::security::authorization::AuthorizationManager;
use crate::web_framework::security::authentication::DelegatingAuthenticationManager;
use crate::web_framework::security::security_filter::SecurityFilterChain;

pub struct HttpSecurity<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static
{
    pub(crate) authentication_manager: Arc<Mutex<Option<DelegatingAuthenticationManager>>>,
    phantom_req: PhantomData<Request>,
    phantom_res: PhantomData<Response>
}

impl <Request, Response> Default for HttpSecurity<Request, Response>
    where
        Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static,
        Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static {
    fn default() -> Self {
        Self {
            authentication_manager: Arc::new(Mutex::new(None)),
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
        SecurityFilterChain { filters: Arc::new(DelegatingFilterProxy::default()) }
    }

    pub fn http() -> HttpSecurity<Request, Response> {
        HttpSecurity::<Request, Response>::default()
    }

    pub fn authorization_manager(&self, authorization_manager: DelegatingAuthenticationManager) {
        *self.authentication_manager.lock().unwrap() = Some(authorization_manager);
    }
}

