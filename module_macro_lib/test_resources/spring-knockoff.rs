#[module_attr]
pub mod test_library {
    pub mod test_library_two {
        use std::fmt::{Debug, Formatter};
        use spring_knockoff_boot_macro::{autowired, bean, service};
        use crate::test_library::test_library_three::One;

        #[derive(Default, Debug)]
        #[service(Once)]
        pub struct Ten {}

        pub mod test_library_six {
            use crate::test_library::test_library_three::Found;

            fn do_it(one: Box<dyn Found>) {}
        }
    }

    pub mod test_library_three {
        use std::sync::{Arc, Mutex};
        use spring_knockoff_boot_macro::{autowired, bean, service};
        use crate::*;

        pub trait Found: Send + Sync {}

        #[service("hello_string")]
        fn this_one() -> Option<&'static str> { Some("hello") }

        impl Found for Four {}

        impl One {
            pub fn one_two_three(&self, one: One) -> String {
                print!("testing...");
                print!("{} is one", one.two.to_string());
                "two one".to_string()
            }
        }

        #[service(Four)]
        #[derive(Default)]
        pub struct Four {
            #[autowired]
            #[mutable_bean] pub one: Arc<Mutex<One>>,
            #[autowired] pub test_one: Arc<One>,
            pub two: String,
        }

        #[service(One)]
        #[derive(Default)]
        pub struct One {
            pub two: String,
        }

        impl Default for Once { fn default() -> Self { Self { a: String::default(), test_dyn_one: Arc::new(Four::default()) as Arc<dyn Found>, test_dyn_one_mutex: Arc::new(Mutex::new(Box::new(Four::default()) as Box<dyn Found>)) } } }

        #[service(Once)]
        pub struct Once {
            #[autowired] pub test_dyn_one: Arc<dyn Found>,
            #[autowired]
            #[mutable_bean] pub test_dyn_one_mutex: Arc<Mutex<Box<dyn Found>>>,
        }

        pub mod test_library_four {
            use crate::test_library::test_library_three::Found;

            #[derive(Default)]
            pub struct TestOneHundred;
        }

        pub mod test_library_five { fn whatever() { pub struct RANDOM; } }
    }
}