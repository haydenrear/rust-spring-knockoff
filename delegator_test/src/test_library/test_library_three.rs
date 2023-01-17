pub trait Found {
}

impl Found for One {
}

impl One {
}

#[derive(Default)]
pub struct Four<'a> {
    one: Option<&'a [String]>
}

#[derive(Default)]
pub struct One {
    pub two: String
}

#[derive(Default)]
pub struct Once {
    pub(crate) fns: Vec<Box<dyn FnOnce(())>>
}

