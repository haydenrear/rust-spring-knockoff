use syn::Item;
use std::env;
use syn::__private::str;
use codegen_utils::syn_helper::SynHelper;
use module_macro_codegen::aspect::AspectParser;
use crate::module_macro_lib::item_parser::item_mod_parser::ItemModParser;
use crate::module_macro_lib::item_parser::ItemParser;
use crate::module_macro_lib::module_parser::{create_initial_parse_container, do_container_modifications};
use crate::module_macro_lib::module_tree::{AutowireType, Bean, BeanDefinitionType, DepType, Profile};
use crate::module_macro_lib::parse_container::ParseContainer;

pub mod module_tree_test;
pub mod profile_tree_test;
pub mod item_parser_test;

fn get_parse_container(module_app: &str, factories: &str) -> Option<ParseContainer> {

    if let Some(Item::Mod(ref mut item_mod)) = get_module_item(module_app, factories) {
        let mut container = ParseContainer::default();
        container.aspects = AspectParser::parse_aspects();

        let module_identifier = item_mod.ident.to_string().clone();

        ItemModParser::parse_item(
            &mut container,
            item_mod,
            vec![module_identifier.clone()],
        );

        return Some(container);
    }

    assert!(false);
    None
}

fn get_container_tup(module_app: &str, factories: &str) -> (Item, (ParseContainer, String)) {
    let module_item = get_module_item(module_app, factories);
    assert!(module_item.is_some());
    let mut module_item = module_item.unwrap();
    let mut parse_container = create_initial_parse_container(&mut module_item);
    assert!(parse_container.is_some());
    let mut container_tup = parse_container.unwrap();
    (module_item, container_tup)
}

fn assert_aspect_info_container(container: &mut ParseContainer) {
    assert_eq!(container.aspects.aspects.len(), 1);

    let mut method_advice_aspects = &mut container.aspects.aspects[0].method_advice_aspects;

    assert_eq!(method_advice_aspects.len(), 2);

    method_advice_aspects.sort();

    let first = &method_advice_aspects[0];
    let second = &method_advice_aspects[1];

    assert_eq!(first.order, 0);
    assert_eq!(second.order, 1);
}


fn get_module_item(module_app: &str, factories: &str) -> Option<Item> {
    set_knockoff_factories(factories);
    let mut syn_file = SynHelper::open_from_base_dir(module_app);
    syn_file.items.get(0).cloned()
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

fn get_abstract_beans(concrete_beans: &Vec<BeanDefinitionType>) -> Vec<(Bean, AutowireType)> {
    concrete_beans.iter().flat_map(|b| {
        match b {
            BeanDefinitionType::Abstract { bean, dep_type } => {
                vec![(bean.clone(), dep_type.clone())]
            }
            BeanDefinitionType::Concrete { bean } => {
                vec![]
            }
        }
    }).collect::<Vec<(Bean, AutowireType)>>()
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

fn get_concrete_beans_with_aspects(container: &mut ParseContainer) -> Vec<Bean> {
    println!("{} is the num injectable types.", container.injectable_types_map.injectable_types.len());
    assert_eq!(container.injectable_types_map.injectable_types.len(), 1);
    get_concrete_beans(&container.injectable_types_map.injectable_types.get(&Profile::default()).unwrap())
        .iter()
        .filter(|b| b.aspect_info.len() != 0)
        .map(|b| b.to_owned())
        .collect::<Vec<Bean>>()
}

fn set_knockoff_factories(module_app: &str) {
    env::var("PROJECT_BASE_DIRECTORY")
        .ok()
        .or(Some("/Users/hayde/IdeaProjects/rust-spring-knockoff/".to_string()))
        .map(|mut p| {
            p = p + module_app;
            env::set_var("KNOCKOFF_FACTORIES", p.as_str());
        });
}
