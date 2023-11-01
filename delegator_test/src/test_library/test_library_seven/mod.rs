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

#[service(TestWithGenerics)]
pub struct TestWithGenerics {
    #[autowired]
    pub f: Arc<TestConstructEnumWithFields>,
}

pub trait TestWithGenericsInTrain<T: Send + Sync>: Send + Sync {
    fn get(&self) -> &T;
}

#[derive(Default)]
#[service(TestGenericsVal)]
pub struct TestGenericsVal;


pub trait SafeVal: Send + Sync {}

#[knockoff_ignore]
impl SafeVal for TestGenericsVal {}

#[service(TestWithGenericsInStruct)]
pub struct TestWithGenericsInStruct {
    #[autowired]
    pub t: Arc<TestGenericsVal>,
}

impl SafeVal for TestWithGenericsInStruct {}

impl TestWithGenericsInTrain<TestGenericsVal> for TestWithGenericsInStruct {
    fn get(&self) -> &TestGenericsVal {
        &self.t
    }
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

#[service(TestT)]
pub struct TestT;
#[service(TestU)]
pub struct TestU;

#[service(TestV)]
pub struct TestV;

// #[service(ContainsPhantom)]
// pub struct ContainsPhantom<T, U, V, Z> {
//     p: PhantomData<T>,
//     u: PhantomData<U>,
//     v: PhantomData<V>,
//     z: PhantomData<Z>
// }
//
// #[service(TestInjectContainsPhantom)]
// pub struct TestInjectContainsPhantom {
//     #[autowired(ContainsPhantom)]
//     contains_phantom: Arc<ContainsPhantom<TestT, TestU, TestV, TestV>>
// }

