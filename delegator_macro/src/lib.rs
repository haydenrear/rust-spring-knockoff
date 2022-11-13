extern crate proc_macro;

use proc_macro::{Span, TokenStream};
use std::collections::{HashMap, LinkedList};
use lazy_static::lazy_static;
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use syn::{DeriveInput, parse_macro_input};
use std::sync::{Arc, Mutex};
use delegator_macro_rules::types;

#[proc_macro_derive(HelperAttr)]
pub fn library(input: TokenStream) -> TokenStream {
    println!("hello!");
    print!("{} is input", input.to_string());
    let input = parse_macro_input!(input as DeriveInput);
    let mut lock = types.values.lock().unwrap();
    lock.insert(String::from("hello"), String::from("goodbye"));
    TokenStream::new()
}
