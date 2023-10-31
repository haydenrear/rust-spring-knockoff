#![feature(proc_macro_hygiene)]

use std::any::TypeId;
use std::fmt::Display;

use quote::{format_ident, IdentFragment, quote, ToTokens};
use syn::{DeriveInput, Field, Fields, Ident, ItemStruct, LitStr, parse_macro_input, Token, token::Paren};
use syn::token::Type;

use serde::{Deserializer};
use module_macro::{module_attr};

use std::any::Any;
use std::sync::{Arc};
use std::collections::HashMap;
use std::ops::Deref;
use std::marker::PhantomData;
use syn::parse::Parser;
use module_macro_lib::module_macro_lib::knockoff_context_builder::bean_constructor_generator::BeanConstructorGenerator;
use module_macro_shared::profile_tree::ProfileBuilder as ModuleProfile;
// these imports are necessary because the generated code does not contain the imports.
include!(concat!(env!("OUT_DIR"), "/spring-knockoff.rs"));


#[module_attr]
#[cfg(springknockoff)]
pub mod test_library {
    use spring_knockoff_boot_macro::*;

    pub mod test_library_two;
    pub use test_library_two::*;

    pub mod test_library_three;
    pub use test_library_three::*;
    pub mod test_library_seven;
    pub use test_library_seven::*;

    #[aspect(test_library.test_library_three.One.*)]
    #[ordered(0)]
    #[cfg(springknockoff)]
    pub fn do_aspect(&self, one: One) -> String {
        println!("hello");
        println!("{}", self.two.clone());
        let found = self.proceed(one);
        let mut three_four = "four three ".to_string() + found.as_str();
        three_four
    }

    #[aspect(test_library.test_library_three.One.*)]
    #[ordered(1)]
    #[cfg(springknockoff)]
    pub fn do_aspect_again(&self, one: One) -> String {
        println!("hello");
        println!("{}", self.two.clone());
        let found = self.proceed(one);
        let mut zero = " zero".to_string();
        zero = found + zero.as_str();
        zero
    }

}

pub use test_library::*;


#[test]
fn test_module_macro() {
    create_with_extra_field();

    let listable: ListableBeanFactory = AbstractListableFactory::<DefaultProfile>::new();
    assert_ne!(listable.singleton_bean_definitions.len(), 0);


    let four_found_again: Option<Arc<TestLibraryFourAgain>> = BeanContainer::<TestLibraryFourAgain>::fetch_bean(&listable);
    assert!(four_found_again.is_some());

    let four_found_again: Option<Arc<Four>> = BeanContainer::<Four>::fetch_bean(&listable);
    four_found_again.unwrap().one.lock().unwrap().two = "another".to_string();

    let one_found: Option<Arc<Ten>> = BeanContainer::<Ten>::fetch_bean(&listable);
    let two_found: Option<Arc<Ten>> = BeanContainer::<Ten>::fetch_bean(&listable);

    assert!(two_found.is_some());
    assert!(one_found.is_some());

    let four_found: Option<Arc<Four>> = BeanContainer::<Four>::fetch_bean(&listable);
    let one_found_again: Option<Arc<One>> = BeanContainer::<One>::fetch_bean(&listable);
    assert!(four_found.as_ref().is_some());
    assert_eq!(four_found.unwrap().one.lock().unwrap().deref().type_id().clone(), one_found_again.as_ref().unwrap().deref().type_id());

    let four_found_third: Option<Arc<Four>> = BeanContainer::<Four>::fetch_bean(&listable);
    assert_eq!(four_found_third.unwrap().one.lock().unwrap().two, "another".to_string());

    let found = BeanContainer::<dyn Found>::fetch_bean(&listable);
    assert!(found.is_some());
    let found = BeanContainerProfile::<dyn Found, DefaultProfile>::fetch_bean_profile(&listable);
    assert!(found.is_some());

    let once_found: Option<Arc<Once>> = BeanContainer::<Once>::fetch_bean(&listable);
    assert!(once_found.is_some());

    let mutable_bean_one = BeanContainer::<Mutex<Box<dyn Found>>>::fetch_bean(&listable);
    assert!(mutable_bean_one.is_some());

    let one = PrototypeBeanContainer::<One>::fetch_bean(&listable);

    assert!(one_found_again.as_ref().is_some());
    let wrapped = one_found_again.unwrap().one_two_three(One { two: "".to_string(), a: "".to_string() });

    assert_eq!(wrapped, "four three two one zero".to_string());

    let with_generics = BeanContainer::<TestWithGenerics>::fetch_bean(&listable);
    assert!(with_generics.is_some(), "Failed to get non-dyn");

    let with_generics = BeanContainer::<dyn HasEnum<TestConstructEnumWithFields>>::fetch_bean(&listable);
    assert!(with_generics.is_some(), "Failed to get dyn");

    let created_enum = BeanContainer::<TestConstructEnumWithFields>::fetch_bean(&listable);
    assert!(created_enum.is_some(), "Failed to create enum.");

}


#[test]
fn test_attribute_handler_mapping() {
    let attr = AttributeHandlerMapping::new();
}

#[test]
fn test_app_ctx() {
    let app_ctx = AppCtx::new();
    assert_eq!(app_ctx.profiles.len(), 1);
    assert!(app_ctx.profiles.iter().any(|p| p == &ModuleProfile::default().profile));
    let found = app_ctx.get_bean::<One>();
    assert!(found.is_some());
    let found = app_ctx.get_bean_for_profile::<One, DefaultProfile>();
    assert!(found.is_some());
    let found = app_ctx.get_bean_for_profile::<Mutex<Box<dyn Found>>, DefaultProfile>();
    assert!(found.is_some());
}

fn create_with_extra_field() {
    let ten = Ten {
        a: String::from("hell")
    };

    let one: One = One {
        a: String::default(),
        two: String::default()
    };

}