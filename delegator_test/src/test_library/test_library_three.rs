use module_macro::{bean, singleton, autowired};
use std::sync::Arc;

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
    four: Arc<One>,
    two: String,
}

#[derive(Default, Debug)]
#[singleton(One)]
pub struct One {
    pub two: String
}

#[singleton(Once)]
#[derive(Default, Debug)]
pub struct Once {
    // pub(crate) fns: Vec<Box<dyn FnOnce(())>>
}

