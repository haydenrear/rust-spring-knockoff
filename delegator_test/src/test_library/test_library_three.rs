use std::sync::{Arc, Mutex};
use spring_knockoff_boot_macro::{autowired, bean, singleton};
use crate::*;

pub mod test_library_four;

pub mod test_library_five;

pub trait Found: Send + Sync {
}

#[singleton("hello_string")]
fn this_one() -> Option<&'static str> {
    Some("hello")
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

#[singleton(Four)]
#[derive(Default)]
pub struct Four {
    #[autowired]
    #[mutable_bean]
    pub one: Arc<Mutex<One>>,
    #[autowired]
    pub test_one: Arc<One>,
    pub two: String,
}

#[singleton(One)]
#[derive(Default)]
pub struct One {
    pub two: String,
}

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

#[singleton(Once)]
pub struct Once {
    #[autowired]
    pub test_dyn_one: Arc<dyn Found>,
    #[autowired]
    #[mutable_bean]
    pub test_dyn_one_mutex: Arc<Mutex<Box<dyn Found>>>,
}

