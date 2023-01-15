#![feature(proc_macro_hygiene)]
use std::fmt::Display;

use quote::{format_ident, IdentFragment, quote, ToTokens};
use syn::{
    DeriveInput,
    Ident,
    LitStr,
    parse_macro_input,
    Token,
    token::Paren
};

use delegator_macro_rules::{last_thing, types};
use module_macro::module_attr;

use crate::test_library::*;
use crate::test_library::test_library_three::{One, Once};
use crate::test_library::test_library_two::Ten;

include!(concat!(env!("OUT_DIR"), "/spring-knockoff.rs"));

#[module_attr]
#[cfg(springknockoff)]
pub mod test_library {

    pub mod test_library_two;

    pub mod test_library_three;

}

#[test]
fn test() {
    let ten = Ten {
        a: String::from("hell")
    };
    let one: One = One{
        a: String::from("hello"),
        two: String::default()
    };
    let mut once = Once {
        a: String::from("hello"),
        fns: vec![Box::new(|()| {
            println!()
        })],
    };
    let container = AppContainer {};
}