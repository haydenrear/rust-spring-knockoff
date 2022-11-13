use delegator_macro::{HelperAttr};
use delegator_macro_rules::{types, last_thing};
use std::fmt::Display;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(HelperAttr)]
#[derive(Debug)]
struct TestStruct {
    field: ()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct() {
        last_thing!();
        let test_struct = TestStruct {field:  ()};
        print!("");
    }
}
