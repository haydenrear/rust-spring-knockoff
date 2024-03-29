use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use quote::ToTokens;
use crate::bean::{BeanDefinition, BeanDefinitionType};
use crate::parse_container::{MetadataItem, MetadataItemId};
use crate::profile_tree::{ProfileBuilder, ProfileTree};
use crate::profile_tree::profile_tree_modifier::ProfileTreeModifier;

use knockoff_logging::*;
use std::sync::Mutex;
use crate::dependency::DepType;
use crate::logger_lazy;
import_logger!("profile_tree.rs");

pub struct ProfileTreeBuilder {
    pub tree_modifiers: Vec<Box<dyn ProfileTreeModifier>>,
    pub injectable_types: HashMap<ProfileBuilder, Vec<BeanDefinitionType>>
}

impl ProfileTreeBuilder {
    pub fn build_profile_tree(
        beans: &mut HashMap<String, BeanDefinition>,
        tree_modifiers: Vec<Box<dyn ProfileTreeModifier>>,
        provided_items: &mut HashMap<MetadataItemId, Vec<Box<dyn MetadataItem>>>
    ) -> ProfileTree
    {

        let mut injectable_types = ProfileTree::create_initial(&beans);

        let mut to_swap = HashMap::new();
        std::mem::swap(&mut to_swap, provided_items);

        let mut profile_tree = ProfileTree {
            injectable_types,
            provided_items: to_swap
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

