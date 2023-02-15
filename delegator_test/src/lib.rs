#![feature(proc_macro_hygiene)]

use std::any::TypeId;
use std::fmt::Display;

use quote::{format_ident, IdentFragment, quote, ToTokens};
use syn::{DeriveInput, Field, Fields, Ident, ItemStruct, LitStr, parse_macro_input, Token, token::Paren};
use syn::token::Type;

use module_macro::{field_aug, initializer, module_attr};

use crate::test_library::*;
use crate::test_library::test_library_three::{One, Once, Four};
use crate::test_library::test_library_two::Ten;

use std::any::Any;
use std::sync::Arc;
use std::collections::HashMap;
use std::ops::Deref;
use std::marker::PhantomData;
use syn::parse::Parser;

include!(concat!(env!("OUT_DIR"), "/spring-knockoff.rs"));

#[module_attr]
#[cfg(springknockoff)]
pub mod test_library {

    pub mod test_library_two;

    pub mod test_library_three;

}

#[initializer]
pub fn example_initializer() {
    println!("hello...");
}

#[field_aug]
pub fn field_aug(struct_item: &mut ItemStruct) {
    match &mut struct_item.fields {
        Fields::Named(ref mut fields_named) => {
            fields_named.named.push(
                Field::parse_named.parse2(quote!(
                                    pub a: String
                                ).into()).unwrap()
            )
        }
        Fields::Unnamed(ref mut fields_unnamed) => {}
        _ => {}
    }
}

#[test]
fn test_module_macro() {
    let ten = Ten {
        a: String::from("hell")
    };

    let one: One = One{
        a: String::default(),
        two: String::default()
    };

    let mut once = Once {
        a: String::from("hello"),
        // fns: vec![Box::new(|()| {
        //     println!()
        // })],
    };

    let listable: ListableBeanFactory = AbstractListableFactory::<DefaultProfile>::new();
    assert_ne!(listable.singleton_bean_definitions.len(), 0);
    let one_found: Option<Arc<Ten>> = listable.get_bean_definition::<Ten>();
    let two_found: Option<Arc<Ten>> = listable.get_bean_definition::<Ten>();
    assert!(one_found.is_some());
    let app_ctx = AppCtx::new();

}