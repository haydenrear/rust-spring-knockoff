use std::sync::{Arc, Mutex};
use spring_knockoff_boot_macro::{autowired, bean, singleton};
use crate::_one_testOne;

pub mod test_library_four;

pub mod test_library_five;

pub trait Found {
}

#[singleton("hello_string")]
fn this_one() -> Option<&'static str> {
    Some("hello")
}

impl Found for One {

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

#[derive(Default, Debug)]
#[singleton(Four)]
pub struct Four {
    #[autowired]
    #[mutable_bean]
    pub one: Arc<Mutex<One>>,
    #[autowired]
    pub test_one: Arc<One>,
    pub two: String,
}

#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[singleton(One)]
pub struct One {
    pub two: String
}

#[singleton(Once)]
#[derive(Default, Debug)]
pub struct Once {
    // pub(crate) fns: Vec<Box<dyn FnOnce(())>>
}

