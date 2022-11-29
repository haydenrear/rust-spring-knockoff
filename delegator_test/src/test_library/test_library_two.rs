use delegator_macro::module_attr;

pub mod test_library_two {
    use crate::test_library::One;

    pub struct Three {
        one: One
    }
}
