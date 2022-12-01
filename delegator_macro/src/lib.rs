extern crate proc_macro;

use delegator_macro_rules::types;
use lazy_static::lazy_static;
use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(HelperAttr)]
pub fn library(input: TokenStream) -> TokenStream {
    println!("hello!");
    print!("{} is input", input.to_string());
    let input = parse_macro_input!(input as DeriveInput);
    let mut lock = types.values.lock().unwrap();
    lock.insert(String::from("hello"), String::from("goodbye"));
    TokenStream::new()
}
