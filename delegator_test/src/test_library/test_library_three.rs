use std::sync::{Arc, Mutex};
use spring_knockoff_boot_macro::{autowired, bean, service, request_body, controller, get_mapping, request_mapping, knockoff_ignore};
use module_precompile_macro::boot_knockoff;

#[boot_knockoff]
pub mod test_library_four {
    use module_precompile_macro::boot_knockoff;
    use spring_knockoff_boot_macro::{autowired, bean, service, request_body, controller, get_mapping, request_mapping, knockoff_ignore};
    use std::sync::{Arc, Mutex};
    use serde::{Serialize, Deserialize};

    #[derive(Default)]
    pub struct TestOneHundred;

    #[service(One)]
    #[derive(Default)]
    pub struct One {
        pub two: String,
    }


    impl One {
        pub fn one_two_three(&self, one: One) -> String {
            print!("testing...");
            print!("{} is one", one.two.to_string());
            "two one".to_string()
        }
    }

    pub trait Found: Send + Sync {
    }

    #[service("hello_string")]
    pub fn this_one() -> FactoryFnTest {
        FactoryFnTest {}
    }

    impl Found for Four {
    }


    #[derive(Serialize, Deserialize, Clone, Default)]
    pub struct ReturnRequest;

    #[request_mapping]
    impl Four {
        #[get_mapping(/v1/dummy/request)]
        pub fn do_request(&self, #[request_body] one: ReturnRequest) -> ReturnRequest {
            one
        }
    }

    #[service(Four)]
    #[derive(Default)]
    #[controller]
    pub struct Four {
        #[autowired]
        #[mutable_bean]
        #[service]
        pub one: Arc<Mutex<One>>,
        #[autowired]
        #[service]
        pub test_one: Arc<One>,
        pub two: String,
    }

    pub struct FactoryFnTest;

    // TODO: implement default with dyn beans.
    impl Default for Once {
        fn default() -> Self {
            Self {
                test_dyn_one: Arc::new(Four::default()) as Arc<dyn Found>,
                test_dyn_one_mutex: Arc::new(Mutex::new(Box::new(Four::default()) as Box<dyn Found>)),
            }
        }
    }

    #[service(Once)]
    pub struct Once {
        #[autowired]
        #[service]
        pub test_dyn_one: Arc<dyn Found>,
        #[autowired]
        #[mutable_bean]
        #[service]
        pub test_dyn_one_mutex: Arc<Mutex<Box<dyn Found>>>,
    }
}

pub use test_library_four::*;

