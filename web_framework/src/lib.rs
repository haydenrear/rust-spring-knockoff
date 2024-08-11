#[macro_use]
extern crate alloc;
extern crate core;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};

pub use authentication_gen::*;

pub mod web_framework {
    pub mod context;
    pub mod request_context;
    pub mod dispatch;
    pub mod convert;
    pub mod filter;
    pub mod http;
    pub mod message;
    pub mod security;
    pub mod session;
    pub mod context_builder;
}

#[test]
fn o() {
    use knockoff_security::JwtToken;;
    let j = AuthenticationType::Jwt( JwtToken{ token: "".to_string() } );
}