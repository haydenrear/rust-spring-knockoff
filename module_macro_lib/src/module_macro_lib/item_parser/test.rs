use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Item, ItemMod, parse_macro_input};
use syn::__private::str;
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::log_message;
use module_macro_codegen::aspect::AspectParser;
use crate::module_macro_lib::item_modifier::delegating_modifier::DelegatingItemModifier;
use crate::module_macro_lib::item_modifier::ItemModifier;
use crate::module_macro_lib::item_parser::item_mod_parser::ItemModParser;
use crate::module_macro_lib::item_parser::ItemParser;
use crate::module_macro_lib::knockoff_context_builder::aspect_generator::AspectGenerator;
use crate::module_macro_lib::module_parser::parse_module;
use crate::module_macro_lib::module_tree::{AspectInfo, AutowireType, Bean, BeanDefinitionType, DepType, Profile};
use crate::module_macro_lib::parse_container::ParseContainer;

#[test]
fn test_parser() {

    set_knockoff_factories();

    let mut lib_file = SynHelper::open_from_base_dir("codegen_resources/spring-knockoff.rs");

    for mut item in lib_file.items {
        let f = match item {
            Item::Mod(ref mut item_mod) => {
                let mut container = ParseContainer::default();
                container.aspects = AspectParser::parse_aspects();

                let module_identifier = item_mod.ident.to_string().clone();

                ItemModParser::parse_item(
                    &mut container,
                    item_mod,
                    vec![module_identifier.clone()]
                );

                Some((container, module_identifier))
            }
            _ => {
                None
            }
        };

        assert!(f.is_some());


        let mut unwrapped = f.unwrap();
        let container = &mut unwrapped.0;

        container.build_to_token_stream();

        assert_eq!(container.aspects.aspects.len(), 1);

        let mut method_advice_aspects = &mut container.aspects.aspects[0].method_advice_aspects;

        assert_eq!(method_advice_aspects.len(), 2);

        method_advice_aspects.sort();

        let first = &method_advice_aspects[0];
        let second = &method_advice_aspects[1];

        assert_eq!(first.order, 0);
        assert_eq!(second.order, 1);

        assert_eq!(container.injectable_types_map.injectable_types.len(), 1);

        let bean_defs = container.injectable_types_map.injectable_types.get(&Profile::default());
        assert!(bean_defs.is_some());

        let concrete_bean_types = get_concrete_bean_types(bean_defs);
        assert_eq!(concrete_bean_types.len(), 4);

        let concrete_beans = get_concrete_beans(bean_defs.unwrap());
        let one_num_deps = get_deps_map(concrete_beans, "Four");

        assert_eq!(one_num_deps.len(), 2);

    }

}

fn get_deps_map(concrete_beans: Vec<Bean>, bean_id: &str) -> Vec<DepType> {
    let one_num_deps = concrete_beans.iter()
        .filter(|b| b.id == bean_id.to_string())
        .map(|b| b.deps_map.clone())
        .next()
        .unwrap();
    one_num_deps
}

fn get_concrete_beans(concrete_beans: &Vec<BeanDefinitionType>) -> Vec<Bean> {
    concrete_beans.iter().flat_map(|b| {
        match b {
            BeanDefinitionType::Abstract { bean, dep_type } => {
                vec![]
            }
            BeanDefinitionType::Concrete { bean } => {
                vec![bean.clone()]
            }
        }
    }).collect::<Vec<Bean>>()
}

fn get_concrete_bean_types(bean_defs: Option<&Vec<BeanDefinitionType>>) -> Vec<&BeanDefinitionType> {
    bean_defs.as_ref().unwrap().iter()
        .filter(|b| {
            match b {
                BeanDefinitionType::Abstract { .. } => {
                    false
                }
                BeanDefinitionType::Concrete { .. } => {
                    true
                }
            }
        }).collect::<Vec<&BeanDefinitionType>>()
}

fn set_knockoff_factories() {
    env::var("PROJECT_BASE_DIRECTORY")
        .ok()
        .or(Some("/Users/hayde/IdeaProjects/rust-spring-knockoff/".to_string()))
        .map(|mut p| {
            p = p + "codegen_resources/multiple_aspects_test.rs";
            env::set_var("KNOCKOFF_FACTORIES", p.as_str());
        });
}

