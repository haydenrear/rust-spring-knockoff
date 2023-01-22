use module_macro::{bean, singleton, Component};

pub trait Found {
}

#[bean("hello_string")]
#[singleton]
fn this_one() -> Option<&'static str> {
    Some("hello")
}

impl Found for One {
}

impl Found for Four {
}

impl One {
}

#[derive(Default)]
#[Component]
pub struct Four {
    two: String
}

#[derive(Default)]
pub struct One {
    pub two: String
}

#[derive(Default)]
pub struct Once {
    pub(crate) fns: Vec<Box<dyn FnOnce(())>>
}

