use std::collections::HashMap;
use quote::ToTokens;
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::module_tree::Bean;
use crate::module_macro_lib::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use crate::module_macro_lib::profile_tree::ProfileTree;
use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub struct MutableFieldTypesArgs {
    mutable_field_types: Vec<String>
}

pub struct MutableProfileTreeModifier {
    bean_id_types: MutableFieldTypesArgs
}

impl ProfileTreeModifier<MutableFieldTypesArgs> for MutableProfileTreeModifier {

    fn create_arg(profile_tree_items: &HashMap<String,Bean>)  -> MutableFieldTypesArgs {
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

    fn modify_bean(&self, d: &mut Bean, profile_tree: &mut ProfileTree) {
        self.bean_id_types.mutable_field_types.iter().for_each(|i| {
            log_message!("Making {} mutable field.", i.as_str());
            let id = d.id.as_str();
            d.deps_map.iter_mut()
                .filter(|d| d.bean_info.type_of_field.to_token_stream().to_string() == i.clone())
                .for_each(|d| {
                    if d.bean_info.mutable {
                        log_message!("Dep type {} is already mutable.", SynHelper::get_str(d.bean_info.field.clone()).as_str());
                    } else {
                        log_message!("Making {} mutable field for {}.", d.bean_info.type_of_field.to_token_stream().to_string().as_str(), id);
                        d.bean_info.mutable = true;
                    }
                });
        });

    }

    fn new(profile_tree_items: &HashMap<String, Bean>) -> Self {
        Self {
            bean_id_types: Self::create_arg(profile_tree_items),
        }
    }
}
