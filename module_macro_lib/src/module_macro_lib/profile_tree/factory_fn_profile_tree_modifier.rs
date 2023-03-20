use std::collections::HashMap;
use syn::{parse2, Type};
use quote::{quote, ToTokens};
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::functions::ModulesFunctions;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub struct FactoryFnProfileTreeModifierArg {
    factory_fn_types: HashMap<String, ModulesFunctions>
}

pub struct FactoryFnProfileTreeModifier {
    beans_to_types: FactoryFnProfileTreeModifierArg,
}

impl ProfileTreeModifier for FactoryFnProfileTreeModifier {

    // TODO: Add abstract traits_impl to beans from qualifiers for the factory_fn so that a BeanFactory
    //  will be made for each abstract and concrete type expected.
    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        todo!()
    }

    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self
        where Self: Sized
    {
        // let factory_fn_types = profile_tree_items.values()
        //     .flat_map(|b| &b.deps_map)
        //     .flat_map(|d| d.item_fn.as_ref()
        //         .map(|i| vec![(d, i)])
        //         .or(Some(vec![])).unwrap()
        //     )
        //     .map(|module_fn| {
        //         module_fn.0.bean_info.qualifier.as_ref().map(|q| {
        //             (q.to_string(), module_fn.1.clone())
        //         }).or_else(|| {
        //             Some((module_fn.1.fn_found.item_fn.sig.ident.to_string(), module_fn.1.clone()))
        //         })
        //     })
        //     .flat_map(|o| o
        //         .map(|o| vec![o])
        //         .or(Some(vec![])).unwrap()
        //     )
        //     .collect::<HashMap<String, ModulesFunctions>>();
        Self {
            beans_to_types: FactoryFnProfileTreeModifierArg {
                factory_fn_types: HashMap::new()
            }
        }
    }
}
