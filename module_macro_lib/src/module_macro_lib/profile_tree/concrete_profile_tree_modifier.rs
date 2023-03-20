use std::collections::HashMap;
use syn::{parse2, Type};
use quote::{quote, ToTokens};
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;

use knockoff_logging::{initialize_log, use_logging};
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub struct AddConcreteTypesToBeansArgs {
    beans_to_types: HashMap<String, Type>,
    bean_struct_ids: Vec<Type>
}

pub struct ConcreteTypeProfileTreeModifier {
    beans_to_types: AddConcreteTypesToBeansArgs,
}

impl ProfileTreeModifier for ConcreteTypeProfileTreeModifier {

    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        dep_type.deps_map.iter_mut()
            .for_each(|dep_type_to_test| {
                if !self.beans_to_types.bean_struct_ids.iter().any(|bean_struct_id|
                    dep_type_to_test.type_path()
                        .as_ref()
                        .filter(|dep_type_bean_path| dep_type_bean_path.bean_path_part_matches(bean_struct_id))
                        .is_some()
                ) {
                    dep_type_to_test.set_abstract();
                    dep_type_to_test.clone().maybe_qualifier().as_ref()
                        .map(|q| self.beans_to_types.beans_to_types.get(q)
                            .map(|type_to_set| dep_type_to_test.set_concrete_field_type(type_to_set.clone()))
                        )
                        .or_else(|| {
                            let bean_path = dep_type_to_test.type_path()
                                .as_ref().map(|b| b.get_inner_type_id());
                            bean_path.map(|b|
                                    self.beans_to_types.beans_to_types.get(&b)
                                        .map(|inner_type| dep_type_to_test.set_concrete_field_type(inner_type.clone()))
                                );
                            None
                        });
                }
            })
    }

    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self {
        Self {
            beans_to_types: Self::create_arg(profile_tree_items)
        }
    }
}

impl ConcreteTypeProfileTreeModifier {

     fn create_arg(profile_tree_items: &HashMap<String, BeanDefinition>) -> AddConcreteTypesToBeansArgs {
         AddConcreteTypesToBeansArgs {

             beans_to_types: profile_tree_items.iter().flat_map(|b| {
                 b.1.traits_impl.iter()
                     .flat_map(|t| BeanFactoryInfo::get_abstract_type(t)
                         .as_ref()
                         .map(|trait_impl| vec![trait_impl.clone()])
                         .or(Some(vec![]))
                         .unwrap()
                     )
                     .map(|t| (t.to_token_stream().to_string(), b.1.struct_type.as_ref().unwrap().clone()))
             }).collect::<HashMap<String, Type>>(),

             bean_struct_ids:  profile_tree_items.values()
                 .flat_map(|s| s.struct_type.as_ref()
                     .map(|s| vec![s.clone()])
                     .or(s.ident.as_ref()
                         .map(|i| i.to_token_stream().to_string().clone()).map(|t| {
                         let type_created = quote! { t };
                         parse2::<Type>(type_created)
                             .ok()
                             .map(|t| vec![t])
                             .or(Some(vec![]))
                             .unwrap()
                     })
                         .or(Some(vec![])))
                     .unwrap()
                 )
                 .collect::<Vec<Type>>()
         }
     }

 }