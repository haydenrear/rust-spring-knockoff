#![feature(str_split_remainder)]
#![feature(async_fn_in_trait)]
#[macro_use]
extern crate alloc;
extern crate core;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};


pub mod web_framework {
    pub mod context;
    pub mod dispatch;
    pub mod convert;
    pub mod filter;
    pub mod http;
    pub mod message;
    pub mod request;
    pub mod security;
    pub mod session;
    pub mod context_builder;
}
