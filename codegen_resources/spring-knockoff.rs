#[module_attr]
pub mod test_library {
    pub mod test_library_two {
        use spring_knockoff_boot_macro::{autowired, bean, singleton};

        #[derive(Default, Debug)]
        #[singleton(Once)]
        pub struct Ten {}
    }

    pub mod test_library_three {
        use std::sync::Arc;
        use spring_knockoff_boot_macro::{autowired, bean, singleton};

        pub trait Found {}

        #[singleton("hello_string")]
        fn this_one() -> Option<&'static str> { Some("hello") }

        impl Found for One {}

        impl Found for Four {}

        impl One {}

        #[derive(Default, Debug)]
        #[singleton(Four)]
        pub struct Four {
            #[autowired] pub one: Arc<One>,
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

        pub mod test_library_four { pub struct One; }

        pub mod test_library_five { pub struct Two; }
    }
}