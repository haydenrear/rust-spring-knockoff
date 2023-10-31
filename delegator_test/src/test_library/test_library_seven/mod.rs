use std::marker::PhantomData;
use spring_knockoff_boot_macro::{service, autowired, enum_service, knockoff_ignore};
use serde::{Deserialize, Serialize};

#[service(TestLibraryFourAgain)]
#[derive(Default)]
pub struct TestLibraryFourAgain;

impl TestLibraryFourAgain {
    pub fn some_test() {
        println!("hello!");
    }
}

pub enum TestConstructEnum {
    One, Two, Three
}

impl TestConstructEnum {
    pub fn new() -> Self{
        TestConstructEnum::One
    }
}

#[enum_service(TestConstructEnumWithFields)]
pub enum TestConstructEnumWithFields {
    One{
        one: String
    }, Two{
        one: String
    }, Three{
        one: String
    }
}

impl Default for TestConstructEnumWithFields {
    fn default() -> Self {
        TestConstructEnumWithFields::One {one: String::default()}
    }
}

pub trait TestDeser: Serialize + for<'a> Deserialize<'a> {
    fn out();
}

#[derive(Serialize, Deserialize, Debug)]
#[knockoff_ignore]
pub struct SomeTest {
    one: String
}


#[knockoff_ignore]
impl TestDeser for SomeTest {
    fn out() {
        println!("Hello world!");
    }
}

pub trait HasEnum<T: Send + Sync>: Send + Sync  {
    fn do_something(&self);
}

use std::sync::Arc;

#[service(TestWithGenerice)]
pub struct TestWithGenerics {
    #[autowired]
    pub f: Arc<TestConstructEnumWithFields>,
}


impl Default for TestWithGenerics {
    fn default() -> Self {
        Self {
            f: TestConstructEnumWithFields::One {
                one: String::default()
            }.into(),
            a: Default::default()
        }
    }
}

impl HasEnum<TestConstructEnumWithFields> for TestWithGenerics {
    fn do_something(&self) {
        println!("Hello!")
    }
}




