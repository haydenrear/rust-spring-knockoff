use std::sync::{Arc, Mutex};
use spring_knockoff_boot_macro::{autowired, bean, service, request_body, controller, get_mapping, request_mapping};
use crate::*;

pub mod test_library_four;

pub mod test_library_five;

pub trait Found: Send + Sync {
}

#[service("hello_string")]
pub fn this_one() -> FactoryFnTest {
    FactoryFnTest {}
}

impl Found for Four {
}

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
    pub one: Arc<Mutex<One>>,
    #[autowired]
    pub test_one: Arc<One>,
    pub two: String,
}

#[service(One)]
#[derive(Default)]
pub struct One {
    pub two: String,
}

pub struct FactoryFnTest;

// TODO: implement default with dyn beans.
impl Default for Once {
    fn default() -> Self {
        Self {
            a: String::default(),
            test_dyn_one: Arc::new(Four::default()) as Arc<dyn Found>,
            test_dyn_one_mutex: Arc::new(Mutex::new(Box::new(Four::default()) as Box<dyn Found>)),
        }
    }
}

#[service(Once)]
pub struct Once {
    #[autowired]
    pub test_dyn_one: Arc<dyn Found>,
    #[autowired]
    #[mutable_bean]
    pub test_dyn_one_mutex: Arc<Mutex<Box<dyn Found>>>,
}

