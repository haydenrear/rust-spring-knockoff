#![feature(proc_macro_hygiene)]

use std::any::TypeId;
use std::fmt::Display;

use quote::{format_ident, IdentFragment, quote, ToTokens};
use syn::{DeriveInput, Field, Fields, Ident, ItemStruct, LitStr, parse_macro_input, Token, token::Paren};
use syn::token::Type;

use module_macro::{module_attr};

use crate::test_library::*;
use crate::test_library::test_library_three::{Four, Once, One};
use crate::test_library::test_library_two::Ten;

use std::any::Any;
use std::sync::Arc;
use std::collections::HashMap;
use std::ops::Deref;
use std::marker::PhantomData;
use syn::parse::Parser;
use spring_knockoff_boot_macro::initializer;

include!(concat!(env!("OUT_DIR"), "/spring-knockoff.rs"));

#[module_attr]
#[cfg(springknockoff)]
pub mod test_library {

    pub mod test_library_two;

    pub mod test_library_three;

}


#[test]
fn test_module_macro() {
    create_with_extra_field();

    let listable: ListableBeanFactory = AbstractListableFactory::<DefaultProfile>::new();
    assert_ne!(listable.singleton_bean_definitions.len(), 0);

    let one_found: Option<Arc<Ten>> = listable.get_bean_definition::<Ten>();
    let two_found: Option<Arc<Ten>> = listable.get_bean_definition::<Ten>();
    assert!(two_found.is_some());
    assert!(one_found.is_some());

    let two_found: Option<Arc<Four>> = listable.get_bean_definition::<Four>();
    let one_found_again: Option<Arc<One>> = listable.get_bean_definition::<One>();
    assert!(two_found.is_some());
    assert_eq!(two_found.unwrap().one.deref(), one_found_again.unwrap().deref());

    let app_ctx = AppCtx::new();
}

fn create_with_extra_field() {
    let ten = Ten {
        a: String::from("hell")
    };

    let one: One = One {
        a: String::default(),
        two: String::default()
    };

    let mut once = Once {
        a: String::from("hello"),
    };
}