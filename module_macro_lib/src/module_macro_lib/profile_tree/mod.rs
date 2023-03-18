use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse2, Type};
use codegen_utils::syn_helper::SynHelper;
use concrete_profile_tree_modifier::ConcreteTypeProfileTreeModifier;
use crate::module_macro_lib::module_tree::InjectableTypeKey;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::bean::{Bean, BeanDefinitionType, BeanPath, BeanPathParts, BeanType};
use module_macro_shared::dependency::{AutowireType, DepType};
use module_macro_shared::profile_tree::{ProfileBuilder, ProfileTree};
use mutable_profile_tree_modifier::MutableProfileTreeModifier;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::profile_tree::profile_profile_tree_modifier::ProfileProfileTreeModifier;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;

pub mod profile_tree_modifier;
pub mod mutable_profile_tree_modifier;
pub mod concrete_profile_tree_modifier;
pub mod profile_profile_tree_modifier;

pub struct ProfileTreeBuilder {
    pub tree_modifiers: Vec<Box<dyn ProfileTreeModifier>>,
    pub injectable_types: HashMap<ProfileBuilder, Vec<BeanDefinitionType>>
}

impl ProfileTreeBuilder {
    pub fn build_profile_tree(beans: &mut HashMap<String, Bean>, tree_modifiers: Vec<Box<dyn ProfileTreeModifier>>)
        -> ProfileTree
    {

        let mut injectable_types = ProfileTree::create_initial(&beans);

        let mut profile_tree = ProfileTree {
            injectable_types,
        };

        let default_profile = ProfileBuilder::default();

        log_message!("{} is the number of beans parsed in profile tree.", beans.len());

        for mut i_type in beans.iter_mut() {

            tree_modifiers.iter()
                .for_each(|t| t.modify_bean(i_type.1, &mut profile_tree));

            log_message!("Adding {} to type.", i_type.1.id.clone());

        }

        log_message!("{:?} is the debugged profile tree.", &profile_tree);
        log_message!("{} is the number after.", profile_tree.injectable_types.get(&default_profile).unwrap().len());

        profile_tree
    }

}

