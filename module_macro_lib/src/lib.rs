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
    use codegen_utils::parse::open_factories_file_syn;
    use codegen_utils::syn_helper::SynHelper;
    use crate::module_macro_lib::module_parser::{parse_item_recursive, parse_module};
    use crate::module_macro_lib::module_tree::{BeanDefinitionType, Profile};
    use crate::module_macro_lib::parse_container::ParseContainer;
    use super::*;

    #[test]
    fn it_works() {
        let lib_file = open_factories_file_syn()
            .expect("Could not open factories file.");

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
        println!("{} is the number of profiles", p.injectable_types_map.injectable_types.len());
        p.injectable_types_map.injectable_types.values()
            .for_each(|i| {
                println!("{} is the number of bean definitions.", i.len());
                assert_eq!(i.len(), 3);
                i.iter().for_each(|b| {
                    match b {
                        BeanDefinitionType::Abstract { bean, dep_type } => {
                            println!("{} is the abstract bean type.", SynHelper::get_str(dep_type.item_impl.trait_.clone().unwrap().1));
                            println!("{} is the number of autowire types for abstract.", bean.traits_impl.len());
                            println!("{} is the number of deps for abstract.", bean.deps_map.len());
                        }
                        BeanDefinitionType::Concrete { bean } => {
                            assert_eq!(bean.traits_impl.len(), 2);
                            assert_eq!(bean.profile.len(), 0);
                            println!("{} is the number of trait types.", bean.traits_impl.len());
                            println!("{} is the number of deps.", bean.deps_map.len());
                        }
                    }
                })
            });
        let keys = p.injectable_types_map.injectable_types.keys()
            .map(|p| p.profile.clone())
            .collect::<Vec<String>>();
        assert_eq!(keys[0], Profile::default().profile);
        let ordering = p.is_valid_ordering_create();
        println!("{} is the ordering", ordering.join(",").as_str())

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
