use std::marker::PhantomData;
use crate::web_framework::security::authentication::AuthenticationToken;

//TODO: singleton of these can be added to each bean that has secured annotation and then create
//  decorator for method and add check if authenticated first, and if so go on to execute block.
pub trait AuthorizationManager<T: AuthorizationObject> {
    fn check(&self, authentication: AuthenticationToken, to_check: &T) -> AuthorizationDecision;
}

pub struct AuthorityAuthorizationManager<T: AuthorizationObject> {
    pub authorities: Vec<String>,
    pub authorization_object: PhantomData<T>
}

impl <T: AuthorizationObject> AuthorizationManager<T> for AuthorityAuthorizationManager<T> {
    fn check(&self, authentication: AuthenticationToken, to_check: &T) -> AuthorizationDecision {
        if authentication.authorities.iter()
            .any(|authority| self.authorities.contains(&authority.authority)) {
            return AuthorizationDecision {granted: true};
        }
        AuthorizationDecision {granted: false}
    }
}

pub trait AuthorizationObject {
}

pub struct AuthorizationDecision {
    pub granted: bool
}
