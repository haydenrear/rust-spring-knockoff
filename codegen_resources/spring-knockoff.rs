#[module_attr]
pub mod test_library {
    pub mod test_library_two {
        use spring_knockoff_boot_macro::{autowired, bean, singleton};

        #[derive(Default, Debug)]
        #[singleton(Once)]
        pub struct Ten {}

        pub mod test_library_six {
            use crate::test_library::test_library_three::Found;

            fn do_it(one: Box<dyn Found>) {}
        }
    }

    pub mod test_library_three {
        use std::sync::{Arc, Mutex};
        use spring_knockoff_boot_macro::{autowired, bean, singleton};
        use crate::_one_testOne;

        pub trait Found {}

        #[singleton("hello_string")]
        fn this_one() -> Option<&'static str> { Some("hello") }

        impl Found for One {}

        impl Found for Four {}

        impl One {
            fn one_two_three(&self, one: One, two: One) -> String {
                print!("testing...");
                print!("{} is one", one.two.to_string());
                "".to_string()
            }
        }

        #[derive(Default, Debug)]
        #[singleton(Four)]
        pub struct Four {
            #[autowired]
            #[mutable_bean] pub one: Arc<Mutex<One>>,
            #[autowired] pub test_one: Arc<One>,
            pub two: String,
        }

        #[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq)]
        #[singleton(One)]
        pub struct One {
            pub two: String,
        }

        #[singleton(Once)]
        #[derive(Default, Debug)]
        pub struct Once {}

        pub mod test_library_four { use crate::test_library::test_library_three::Found; }

        pub mod test_library_five { fn whatever() { pub struct RANDOM; } }
    }
}
