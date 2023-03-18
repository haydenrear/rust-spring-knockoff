use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use web_framework_shared::matcher::{AntPathRequestMatcher, Matcher};
use web_framework_shared::request::{AuthorizationObject, WebRequest};
use crate::web_framework::context_builder::DelegatingAuthenticationManagerBuilder;
use crate::web_framework::convert::{Register, Registration};
use crate::web_framework::security::authentication::{AuthenticationProvider, AuthenticationToken};

//TODO: singleton of these can be added to each bean that has secured annotation and then create
//  decorator for method and add check if authenticated first, and if so go on to execute block.
pub trait AuthorizationManager<T: AuthorizationObject> {
    fn check(&self, authentication: &AuthenticationToken, to_check: &T) -> AuthorizationDecision;
}

#[derive(Clone, Default)]
pub struct AuthorityAuthorizationManager<T: AuthorizationObject> {
    pub authorities: Vec<String>,
    pub authorization_object: PhantomData<T>
}

impl <T: AuthorizationObject> AuthorizationManager<T> for AuthorityAuthorizationManager<T> {
    fn check(&self, authentication: &AuthenticationToken, to_check: &T) -> AuthorizationDecision {
        if authentication.authorities.iter()
            .any(|authority| self.authorities.contains(&authority.authority)) {
            return AuthorizationDecision { granted: true };
        }
        AuthorizationDecision { granted: false }
    }
}

#[derive(Default, Clone)]
pub struct RequestMatcherDelegatingAuthorizationManager {
    pub authority_authorization_managers: Vec<RequestMatcherEntry<AuthorityAuthorizationManager<WebRequest>>>
}

#[derive(Default, Clone)]
pub struct DelegatingAuthorizationManagerBuilder {
    pub authority_authorization_managers: Arc<Mutex<Vec<RequestMatcherEntry<AuthorityAuthorizationManager<WebRequest>>>>>,
}

impl DelegatingAuthorizationManagerBuilder {
    pub fn build(&self) -> RequestMatcherDelegatingAuthorizationManager {
        let mut guard = self.authority_authorization_managers.as_ref().lock().unwrap();
        let mut next = vec![];
        std::mem::swap(&mut next, guard.as_mut());
        RequestMatcherDelegatingAuthorizationManager {
            authority_authorization_managers: next,
        }
    }

    pub fn new() -> Self {
        Self {
            authority_authorization_managers: Arc::new(Mutex::new(vec![]))
        }
    }
}

impl Register<RequestMatcherEntry<AuthorityAuthorizationManager<WebRequest>>> for DelegatingAuthorizationManagerBuilder {
    fn register(&self, mut auth: RequestMatcherEntry<AuthorityAuthorizationManager<WebRequest>>) {
        self.authority_authorization_managers.as_ref().lock().unwrap().push(auth)
    }
}

#[derive(Default, Clone)]
pub struct RequestMatcherEntry<T: AuthorizationManager<WebRequest>> {
    entry: T,
    ant_path_request_matcher: Vec<AntPathRequestMatcher>
}

impl RequestMatcherEntry<AuthorityAuthorizationManager<WebRequest>> {
    pub fn new(endpoints: Vec<&str>, authorities: Vec<&str>) -> Self {
        let ant_path_request_matcher = endpoints.iter()
            .map(|e| AntPathRequestMatcher::new(e, "/"))
            .collect::<Vec<AntPathRequestMatcher>>();
         let authorities = authorities.iter()
             .map(|authority| authority.to_string())
             .collect::<Vec<String>>();
        let entry = AuthorityAuthorizationManager {
            authorities,
            authorization_object: Default::default(),
        };
        RequestMatcherEntry {
            entry,
            ant_path_request_matcher,
        }
    }
}

impl <T: AuthorizationManager<WebRequest>> RequestMatcherEntry<T> {
    pub fn matches(&self, web_request: &WebRequest) -> bool {
        self.ant_path_request_matcher.iter().any(|ant_path_matcher| ant_path_matcher.matches(&web_request))
    }
}

impl AuthorizationManager<WebRequest> for RequestMatcherDelegatingAuthorizationManager {
    fn check(&self, authentication: &AuthenticationToken, to_check: &WebRequest) -> AuthorizationDecision {
        self.authority_authorization_managers.iter()
            .filter(|a| a.matches(to_check))
            .map(|a| a.entry.check(authentication, to_check))
            .next()
            .or(Some(AuthorizationDecision { granted: false }))
            .unwrap()
    }
}

pub struct AuthorizationDecision {
    pub granted: bool
}
