use std::sync::Arc;
use spring_knockoff_boot_macro::{autowired, bean, singleton};

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
}

#[derive(Default, Debug)]
#[singleton(Four)]
pub struct Four {
    #[autowired]
    pub one: Arc<One>,
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

