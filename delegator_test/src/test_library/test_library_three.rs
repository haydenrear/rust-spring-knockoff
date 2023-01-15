pub trait Found {
}

impl Found for One {
}

impl One {
    // fn new() -> Self {
    //     Self {
    //         a: String::from(""),
    //         two: String::default()
    //     }
    // }
}

pub struct Four<'a> {
    one: &'a [String]
}

pub struct One {
    pub(crate) two: String
}

pub struct Once {
    pub(crate) fns: Vec<Box<dyn FnOnce(())>>
}
