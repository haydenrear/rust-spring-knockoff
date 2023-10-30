use std::collections::HashMap;
use quote::ToTokens;
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("mutable_profile_tree_modifier.rs");

pub struct MutableFieldTypesArgs {
    mutable_field_types: Vec<String>
}

pub struct MutableProfileTreeModifier {
    bean_id_types: MutableFieldTypesArgs
}

impl ProfileTreeModifier for MutableProfileTreeModifier {

    fn modify_bean(&self, d: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        self.bean_id_types.mutable_field_types.iter().for_each(|i| {
            log_message!("Making {} mutable field.", i.as_str());
            let id = d.id.as_str();
            d.deps_map.iter_mut()
                .filter(|d| d.identifier() == i.clone())
                .for_each(|d| {
                    if d.mutable() {
                        log_message!("Dep type {} is already mutable.", SynHelper::get_str(d.identifier()));
                    } else {
                        log_message!("Making {} mutable field for {}.", d.identifier(), id);
                        d.set_mutable();
                    }
                });
        });
    }

    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self {
        Self {
            bean_id_types: Self::create_arg(profile_tree_items),
        }
    }
}

impl MutableProfileTreeModifier {
    fn create_arg(profile_tree_items: &HashMap<String, BeanDefinition>) -> MutableFieldTypesArgs {
        MutableFieldTypesArgs {
            mutable_field_types: profile_tree_items.values()
                .filter(|b| b.mutable)
                .flat_map(|b| {
                    b.ident.as_ref().map(|i| i.to_token_stream().to_string().clone())
                        .or_else(|| b.struct_type.as_ref().map(|s| s.to_token_stream().to_string().clone()))
                        .map(|a| vec![a])
                        .or(Some(vec![]))
                        .unwrap()
                })
                .collect::<Vec<String>>()
        }
    }
}