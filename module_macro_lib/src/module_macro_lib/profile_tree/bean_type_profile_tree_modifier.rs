use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use syn::{parse2, Type};
use quote::{quote, ToTokens};
use module_macro_shared::bean::{AbstractionLevel, BeanDefinition, BeanType};
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;

use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use module_macro_shared::dependency::DepType;
use crate::logger_lazy;
use crate::module_macro_lib::profile_tree::search_profile_tree::SearchProfileTree;
import_logger!("concrete_profile_tree_modifier.rs");


#[derive(Eq, PartialOrd, PartialEq, Ord)]
pub struct BeanFieldTypes {
    type_names: Vec<String>
}

pub struct NoOpBeanArgs {
    profile_tree_items: HashMap<String, BeanDefinition>
}

pub struct BeanTypeProfileTreeModifier {
    beans_to_types: NoOpBeanArgs,
}

impl ProfileTreeModifier for BeanTypeProfileTreeModifier {

    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        info!("Doing modify and bean.");
        // let dep_profiles = dep_type.profile.iter().collect();
        // dep_type.deps_map.iter_mut()
        //     .for_each(|dep_type_to_test| {
        //         if !self.beans_to_types.profile_tree_items.iter()
        //             .any(|(bean_struct_id, bean_def)|
        //                 dep_type_to_test.bean_type_path()
        //                     .as_ref()
        //                     .filter(|dep_type_bean_path| dep_type_bean_path.bean_path_part_matches(bean_struct_id))
        //                     .is_some()
        //         ) {
        //             let deps = profile_tree.search_profile_tree(
        //                 dep_type_to_test,
        //                 Some(&dep_profiles),
        //                 None,
        //
        //             );
        //
        //             let bean_type_deps = deps.iter()
        //                 .flat_map(|d| d.bean().bean_type.iter())
        //                 .collect::<Vec<&BeanType>>();
        //
        //             assert!(bean_type_deps.len() <= 1, "Bean should only have one dependency definition supplied per profile!");
        //             let mut bean_dep_type = bean_type_deps.into_iter().next().cloned();
        //
        //             info!("Doing modify and bean structs id was not contained for {:?}.", dep_type_to_test);
        //             dep_type_to_test.set_bean_type(&mut bean_dep_type);
        //         }
        //     })
        todo!()
    }

    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self {
        Self {
            beans_to_types: Self::create_arg(profile_tree_items)
        }
    }
}

impl BeanTypeProfileTreeModifier {

     fn create_arg(profile_tree_items: &HashMap<String, BeanDefinition>) -> NoOpBeanArgs {
         NoOpBeanArgs {
             profile_tree_items: profile_tree_items.clone(),
         }
     }

 }