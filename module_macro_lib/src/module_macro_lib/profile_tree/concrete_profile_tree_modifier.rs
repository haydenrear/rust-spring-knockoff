use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use syn::{parse2, Type};
use quote::{quote, ToTokens};
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;

use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use module_macro_shared::dependency::DepType;
use crate::logger_lazy;
import_logger!("concrete_profile_tree_modifier.rs");


#[derive(Eq, PartialOrd, PartialEq, Ord)]
pub struct BeanFieldTypes {
    type_names: Vec<String>
}

impl Hash for BeanFieldTypes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut to_sort = self.type_names.iter().collect::<Vec<&String>>();
        to_sort.sort();
        to_sort.iter().for_each(|next_value| state.write(next_value.as_bytes()));
    }
}

pub struct AddConcreteTypesToBeansArgs {
    beans_to_types: HashMap<String, Type>,
    bean_struct_ids: Vec<Type>
}

pub struct ConcreteTypeProfileTreeModifier {

    /// TODO: the key here needs to be smarter - the bean dependency parser goes through in the fields of the
    ///     structs and looks for their associated keys to find which types to inject. So there is an edge
    ///     case - which is the generics. So in the generics case, then when a field is retrieved it needs
    ///     to find the trait that impls all of the generics that it has... and then potentially the lifetimes
    ///     at some point. Currently using autowired qualifier value.
    beans_to_types: AddConcreteTypesToBeansArgs,
}

impl ProfileTreeModifier for ConcreteTypeProfileTreeModifier {

    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        info!("Doing modify and bean.");
        dep_type.deps_map.iter_mut()
            .for_each(|dep_type_to_test| {
                if !self.beans_to_types.bean_struct_ids.iter()
                    .any(|bean_struct_id|
                        dep_type_to_test.bean_type_path()
                            .as_ref()
                            .filter(|dep_type_bean_path| dep_type_bean_path.bean_path_part_matches(bean_struct_id))
                            .is_some()
                ) {
                    info!("Doing modify and bean structs id was not contained for {:?}.", dep_type_to_test);
                    dep_type_to_test.set_is_abstract(&mut Some(true));
                    dep_type_to_test.dep_type_maybe_qualifier().clone().as_ref()
                        // Here is where it's getting the struct, setting the concrete field type of the
                        // dependency metadata, which will then be used in the BeanFactoryInfo, when the
                        // factories are being created. See above TODO:
                        .map(|q| self.beans_to_types.beans_to_types.get(q)
                            .map(|type_to_set| dep_type_to_test.bean_info_mut()
                                .set_concrete_type_of_field_bean_type(&mut Some(type_to_set.clone())))
                        )
                        .or_else(|| {
                            info!("Doing modify and {:?} was not in beans to types.", dep_type_to_test);
                            let bean_path = dep_type_to_test.bean_type_path()
                                .as_ref()
                                .map(|b| b.get_inner_type_id());

                            bean_path.map(|b|
                                    self.beans_to_types.beans_to_types.get(&b)
                                        .map(|inner_type| dep_type_to_test.bean_info_mut()
                                            .set_concrete_type_of_field_bean_type(&mut Some(inner_type.clone()))
                                        )
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
                 if b.1.struct_type.is_none() {
                     vec![]
                 } else {
                     b.1.traits_impl.iter()
                         .flat_map(|t| BeanFactoryInfo::get_abstract_type(t)
                             .as_ref()
                             .map(|trait_impl| vec![trait_impl.clone()])
                             .or(Some(vec![]))
                             .unwrap()
                         )
                         .map(|t| (t.to_token_stream().to_string(), b.1.struct_type.clone()
                             .or(b.1.ident.as_ref().map(|i| parse2::<Type>(i.to_token_stream()).ok()).flatten())
                             .unwrap().clone()))
                         .collect::<Vec<_>>()
                 }
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