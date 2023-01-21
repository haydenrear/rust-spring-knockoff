#![feature(pattern)]
pub mod module_macro_lib {
    pub mod app_container;
    pub mod module_parser;
    pub mod module_tree;
    pub mod spring_knockoff_context;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
