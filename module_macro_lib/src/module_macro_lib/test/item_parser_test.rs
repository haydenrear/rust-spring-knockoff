use syn::Item;
use std::env;
use quote::ToTokens;
use syn::__private::str;
use codegen_utils::syn_helper::SynHelper;
use module_macro_codegen::aspect::AspectParser;
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::dependency::FieldDepType;
use module_macro_shared::profile_tree::ProfileBuilder;
use crate::module_macro_lib::item_parser::item_mod_parser::ItemModParser;
use crate::module_macro_lib::item_parser::ItemParser;
use crate::module_macro_lib::module_parser::{create_initial_parse_container, do_container_modifications};
use module_macro_shared::bean::BeanDefinitionType;
use module_macro_shared::parse_container::ParseContainer;
use crate::module_macro_lib::parse_container::ParseContainerBuilder;
use crate::module_macro_lib::test::{assert_aspect_info_container, get_abstract_beans, get_concrete_bean_types, get_concrete_beans, get_concrete_beans_with_aspects, get_container_tup, get_deps_map, get_parse_container};

#[test]
fn test_parse_module() {
    let (mut module_item, mut container_tup) = get_container_tup(
        "module_macro_lib/test_resources/spring-knockoff.rs",
        "module_macro_lib/test_resources/multiple_aspects_test.rs"
    );
    let container = do_container_modifications(&mut module_item, &mut container_tup);
    ParseContainerBuilder::build_to_token_stream(container);
    assert_aspect_info_container(container);
    assert_eq!(container.aspects.aspects[0].method_advice_aspects.len(), 2);
    let beans_with_aspects = get_concrete_beans_with_aspects(container);
    assert_eq!(beans_with_aspects.len(), 1);
    assert_eq!(beans_with_aspects[0].aspect_info.len(), 1);
    assert_eq!(beans_with_aspects[0].aspect_info[0].advice_chain.len(), 1);
}

#[test]
fn test_method_advice_aspects() {
    let container_opt = get_parse_container(
        "module_macro_lib/test_resources/spring-knockoff.rs",
        "module_macro_lib/test_resources/multiple_aspects_test.rs"
    );
    let mut container = container_opt.unwrap();
    assert_aspect_info_container(&mut container);
}

#[test]
fn test_injectable_types() {

    let container_opt = get_parse_container(
        "module_macro_lib/test_resources/spring-knockoff.rs",
        "module_macro_lib/test_resources/multiple_aspects_test.rs"
    );

    assert!(container_opt.is_some());

    let mut container = container_opt.unwrap();

    ParseContainerBuilder::build_to_token_stream(&mut container);

    assert_eq!(container.profile_tree.injectable_types.len(), 1);

    let bean_defs = container.profile_tree.injectable_types.get(&ProfileBuilder::default());
    assert!(bean_defs.is_some());

    let concrete_bean_types = get_concrete_bean_types(bean_defs);
    assert_eq!(concrete_bean_types.len(), 4);

    concrete_bean_types.iter().for_each(|c| {
        match c {
            BeanDefinitionType::Abstract { bean, dep_type } => {

            }
            BeanDefinitionType::Concrete { bean } => {
                println!("Printing use stmt");
                bean.get_use_stmts().values()
                    .for_each(|u| {
                        println!("{} is use stmt.", u.to_token_stream().to_string());
                    })
            }
        }
    });

    let concrete_beans = get_concrete_beans(bean_defs.unwrap());
    let one_beans = concrete_beans.iter()
        .filter(|b| b.id.clone() == "One".to_string())
        .collect::<Vec<&BeanDefinition>>();
    assert_eq!(one_beans.len(), 1);

    let one_num_deps = get_deps_map(concrete_beans, "Four");
    assert_eq!(one_num_deps.len(), 2);

    let abstract_beans = get_abstract_beans(bean_defs.unwrap());
    assert_eq!(abstract_beans.len(), 1);
    assert_eq!(abstract_beans.iter().map(|b| &b.0.id).next().unwrap(), &"Four");


}

