#[module_attr]
pub mod test_library {
    use spring_knockoff_boot_macro::*;
    pub use test_library_two::*;
    pub use test_library_three::*;
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

    pub mod test_library_two {
        use std::fmt::{Debug, Formatter};
        use spring_knockoff_boot_macro::{autowired, bean, enable_http_security, service, qualifier};
        use web_framework::web_framework::security::http_security::HttpSecurity;
        use crate::test_library::test_library_three::One;
        use serde::{Deserialize, Serialize};

        #[derive(Default, Debug)]
        #[service(Once)]
        pub struct Ten {}

        #[enable_http_security]
        pub fn enable_http_security<Request, Response>(http: &mut HttpSecurity<Request, Response>) where Response: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static, Request: Serialize + for<'b> Deserialize<'b> + Clone + Default + Send + Sync + 'static { http.request_matcher(vec!["one", "two"], vec!["authority_one"]); }

        pub mod test_library_six {
            use crate::test_library::test_library_three::Found;

            fn do_it(one: Box<dyn Found>) {}
        }
    }

    pub mod test_library_three {
        use std::sync::{Arc, Mutex};
        use spring_knockoff_boot_macro::{autowired, bean, service, request_body, controller, get_mapping, request_mapping};
        use crate::*;

        pub trait Found: Send + Sync {}

        #[service("hello_string")]
        pub fn this_one() -> FactoryFnTest { FactoryFnTest {} }

        impl Found for Four {}

        impl One {
            pub fn one_two_three(&self, one: One) -> String {
                print!("testing...");
                print!("{} is one", one.two.to_string());
                "two one".to_string()
            }
        }

        #[derive(Serialize, Deserialize, Clone, Default)]
        pub struct ReturnRequest;

        #[request_mapping]
        impl Four {
            #[get_mapping(/ v1 / dummy / request)]
            pub fn do_request(&self, #[request_body] one: ReturnRequest) -> ReturnRequest { one }
        }

        #[service(Four)]
        #[derive(Default)]
        #[controller]
        pub struct Four {
            #[autowired]
            #[mutable_bean]
            #[service] pub one: Arc<Mutex<One>>,
            #[autowired]
            #[service] pub test_one: Arc<One>,
            pub two: String,
        }

        #[service(One)]
        #[derive(Default)]
        pub struct One {
            pub two: String,
        }

        pub struct FactoryFnTest;

        impl Default for Once { fn default() -> Self { Self { a: String::default(), test_dyn_one: Arc::new(Four::default()) as Arc<dyn Found>, test_dyn_one_mutex: Arc::new(Mutex::new(Box::new(Four::default()) as Box<dyn Found>)) } } }

        #[service(Once)]
        pub struct Once {
            #[autowired]
            #[service] pub test_dyn_one: Arc<dyn Found>,
            #[autowired]
            #[mutable_bean]
            #[service] pub test_dyn_one_mutex: Arc<Mutex<Box<dyn Found>>>,
        }

        pub mod test_library_four {
            #[derive(Default)]
            pub struct TestOneHundred;
        }

        pub mod test_library_five { fn whatever() { pub struct RANDOM; } }
    }

    pub mod test_library_seven {
        use std::marker::PhantomData;
        use spring_knockoff_boot_macro::{service, autowired, enum_service, knockoff_ignore, prototype};
        use serde::{Deserialize, Serialize};

        #[service(TestLibraryFourAgain)]
        #[derive(Default)]
        pub struct TestLibraryFourAgain;

        impl TestLibraryFourAgain { pub fn some_test() { println!("hello!"); } }

        pub enum TestConstructEnum { One, Two, Three }

        impl TestConstructEnum { pub fn new() -> Self { TestConstructEnum::One } }

        #[enum_service(TestConstructEnumWithFields)]
        pub enum TestConstructEnumWithFields { One { one: String }, Two { one: String }, Three { one: String } }

        impl Default for TestConstructEnumWithFields { fn default() -> Self { TestConstructEnumWithFields::One { one: String::default() } } }

        pub trait TestDeser: Serialize + for<'a> Deserialize<'a> { fn out(); }

        #[derive(Serialize, Deserialize, Debug)]
        #[knockoff_ignore]
        pub struct SomeTest {
            one: String,
        }

        #[knockoff_ignore]
        impl TestDeser for SomeTest { fn out() { println!("Hello world!"); } }

        pub trait HasEnum<T: Send + Sync>: Send + Sync { fn do_something(&self); }

        use std::sync::Arc;

        #[service(TestWithGenerics)]
        pub struct TestWithGenerics {
            #[autowired]
            #[service] pub f: Arc<TestConstructEnumWithFields>,
        }

        pub trait TestWithGenericsInTrain<T: Send + Sync>: Send + Sync { fn get(&self) -> &T; }

        #[derive(Default)]
        #[service(TestGenericsVal)]
        pub struct TestGenericsVal;

        pub trait SafeVal: Send + Sync {}

        #[knockoff_ignore]
        impl SafeVal for TestGenericsVal {}

        #[service(TestWithGenericsInStruct)]
        pub struct TestWithGenericsInStruct {
            #[autowired]
            #[service] pub t: Arc<TestGenericsVal>,
        }

        impl SafeVal for TestWithGenericsInStruct {}

        impl TestWithGenericsInTrain<TestGenericsVal> for TestWithGenericsInStruct { fn get(&self) -> &TestGenericsVal { &self.t } }

        impl Default for TestWithGenerics { fn default() -> Self { Self { f: TestConstructEnumWithFields::One { one: String::default() }.into(), a: Default::default() } } }

        impl HasEnum<TestConstructEnumWithFields> for TestWithGenerics { fn do_something(&self) { println!("Hello!") } }

        #[service(TestT)]
        #[derive(Default)]
        pub struct TestT;

        #[service(TestU)]
        #[derive(Default)]
        pub struct TestU;

        #[service(TestV)]
        #[derive(Default)]
        pub struct TestV;

        #[derive(Default)]
        pub struct ContainsPhantom<T, U, V, Z> {
            p: PhantomData<T>,
            u: PhantomData<U>,
            v: PhantomData<V>,
            z: PhantomData<Z>,
        }

        #[service(ContainsPhantom)]
        pub fn test_phantom() -> ContainsPhantom<TestT, TestU, TestV, TestV> {
            println!("In test phantom factory!");
            ContainsPhantom::default()
        }

        #[service(TestInjectContainsPhantom)]
        pub struct TestInjectContainsPhantom {
            #[autowired(ContainsPhantom)]
            #[service] pub contains_phantom: Arc<ContainsPhantom<TestT, TestU, TestV, TestV>>,
        }

        #[derive(Default)]
        #[prototype(TestPrototypeBean)]
        pub struct TestPrototypeBean {}

        #[service(TestInjectPrototypeBean)]
        pub struct TestInjectPrototypeBean {
            #[autowired]
            #[prototype] pub test_prototype_bean: TestPrototypeBean,
        }
    }

    pub mod authentication_type {
        use spring_knockoff_boot_macro::*;
        use serde::{Serialize, Deserialize};
        use web_framework_shared::*;
        use knockoff_security::{AuthenticationAware, AuthType};

        #[derive(Default, Clone, Debug, Serialize, Deserialize)]
        #[auth_type_struct(TestAuthType)]
        pub struct TestAuthType {
            some_token: String,
        }

        #[auth_type_impl(TestAuthType)]
        impl AuthType for TestAuthType {
            const AUTH_TYPE: &'static str = "test_auth_type";
            fn parse_credentials(request: &WebRequest) -> Result<Self, AuthenticationConversionError> { Ok(TestAuthType::default()) }
        }

        #[auth_type_aware(TestAuthType)]
        impl AuthenticationAware for TestAuthType {
            fn get_authorities(&self) -> Vec<GrantedAuthority> { todo!() }
            fn get_credentials(&self) -> Option<String> { todo!() }
            fn get_principal(&self) -> Option<String> { todo!() }
            fn set_credentials(&mut self, credential: String) { todo!() }
            fn set_principal(&mut self, principal: String) { todo!() }
        }
    }
}