use std::collections::HashMap;
use syn::{parse2, Type};
use quote::{quote, ToTokens};
use module_macro_shared::bean::Bean;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub struct AddConcreteTypesToBeansArgs {
    beans_to_types: HashMap<String, Type>,
    bean_ids: Vec<Type>
}

pub struct ConcreteTypeProfileTreeModifier {
    beans_to_types: AddConcreteTypesToBeansArgs,
}

impl ProfileTreeModifier for ConcreteTypeProfileTreeModifier {

    fn modify_bean(&self, dep_type: &mut Bean, profile_tree: &mut ProfileTree) {
        dep_type.deps_map.iter_mut()
            .for_each(|d| {
                if self.beans_to_types.bean_ids.iter().any(|b|
                    d.bean_type_path
                        .as_ref()
                        .filter(|d| d.bean_path_part_matches(b))
                        .is_some()
                ) {
                    d.is_abstract = Some(false);
                } else {
                    d.is_abstract = Some(true);
                    d.bean_info.qualifier.as_ref()
                        .map(|q| self.beans_to_types.beans_to_types.get(q)
                        .map(|type_to_set| d.bean_info.concrete_type_of_field_bean_type = Some(type_to_set.clone()))
                    ).or_else(|| {
                        d.bean_type_path.as_ref()
                            .map(|b|
                                self.beans_to_types.beans_to_types.get(&b.get_inner_type_id())
                                    .map(|inner_type| d.bean_info.concrete_type_of_field_bean_type = Some(inner_type.clone()))
                            );
                        None
                    });
                }
            })
    }

    fn new(profile_tree_items: &HashMap<String, Bean>) -> Self {
        Self {
            beans_to_types: Self::create_arg(profile_tree_items)
        }
    }
}

impl ConcreteTypeProfileTreeModifier {

     fn create_arg(profile_tree_items: &HashMap<String, Bean>) -> AddConcreteTypesToBeansArgs {
         AddConcreteTypesToBeansArgs {

             beans_to_types: profile_tree_items.iter().flat_map(|b| {
                 b.1.traits_impl.iter()
                     .flat_map(|t| t.item_impl.trait_.as_ref().map(|trait_impl| vec![trait_impl.1.clone()]).or(Some(vec![])).unwrap())
                     .map(|t| (t.to_token_stream().to_string(), b.1.struct_type.as_ref().unwrap().clone()))
             }).collect::<HashMap<String, Type>>(),

             bean_ids:  profile_tree_items.values()
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