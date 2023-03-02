#![feature(pattern)]

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub mod module_macro_lib {
    pub mod parse_container;
    pub mod module_parser;
    pub mod module_tree;
    pub mod knockoff_context_builder;
    pub mod profile_tree;
    pub mod fn_parser;
    pub mod util;
    pub mod bean_parser;
    pub mod context_builder;
    pub mod initializer;
    pub mod knockoff_context;
    pub mod debug;
    pub mod default_impls;
    pub mod logging;
    pub mod aspect;
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use base64::decode;
    use quote::ToTokens;
    use syn::{Item, parse_macro_input};
    use crate::module_macro_lib::module_parser::{parse_item_recursive, parse_module};
    use crate::module_macro_lib::parse_container::ParseContainer;
    use super::*;

    #[test]
    fn it_works() {
        let lib_file = get_syn_file("test_library_three.rs");

        let items = lib_file.items.clone();
        let mut p = ParseContainer::default();
        for mut item in items {
            match &mut item {
                Item::Mod(ref mut module_found) => {
                    parse_item_recursive(module_found, &mut p, &mut vec![])
                }
                _ => {}
            }
        }

        p.build_injectable();
        let ordering = p.is_valid_ordering_create();
        println!("{}", ordering.join(",").as_str())

    }

    fn get_syn_file(path: &str) -> syn::File {
        let p = ParseContainer::default();

        let mut file_result = File::open(
            Path::new("/Users/hayde/IdeaProjects/rust-spring-knockoff/module_macro_lib/resources")
                .join(path)
        )
            .or_else(|f| {
                Err(())
            });

        let mut file = file_result.unwrap();
        let mut src = String::new();

        file.read_to_string(&mut src)
            .unwrap();
        let mut lib_file = syn::parse_file(&src)
            .unwrap();
        lib_file
    }
}
