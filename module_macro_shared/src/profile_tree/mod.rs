use std::cmp::Ordering;
use syn::{Attribute, Block, Field, Fields, ImplItemMethod, ItemEnum, ItemImpl, ItemStruct, Lifetime, Path, Stmt, Type, TypeArray};
use quote::ToTokens;
use std::fmt::{Debug, Formatter};
use std::fmt;
use codegen_utils::syn_helper;
use syn::__private::str;
use proc_macro2::Ident;
use codegen_utils::syn_helper::SynHelper;

use std::collections::HashMap;
use std::sync::Arc;
use crate::bean::{BeanDefinition, BeanDefinitionType};
use crate::dependency::{DependencyDescriptor, DependencyMetadata};
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("profile_tree.rs");
use crate::parse_container::{MetadataItem, MetadataItemId};

pub mod profile_tree_modifier;
pub mod profile_tree_finalizer;

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash, Debug)]
pub struct ProfileBuilder {
    pub profile: String,
}

impl Default for ProfileBuilder {
    fn default() -> Self {
        Self {
            profile: "DefaultProfile".to_string()
        }
    }
}

#[derive(Default, Debug)]
pub struct ProfileTree {
    /// for profile implementations.
    pub injectable_types: HashMap<ProfileBuilder, Vec<BeanDefinitionType>>,
    pub provided_items: HashMap<MetadataItemId, Vec<Box<dyn MetadataItem>>>
}


impl Clone for ProfileTree {
    fn clone(&self) -> Self {
        Self {
            injectable_types: self.injectable_types.clone(),
            provided_items: HashMap::new()
        }
    }
}

impl ProfileTree {

    pub fn add_to_profile_concrete(&mut self, i_type: &BeanDefinition, profile: &ProfileBuilder) {
        log_message!("Adding {} to {} profiles.", &i_type.id, profile.profile.as_str());
        self.injectable_types.get_mut(profile)
            .map(|beans_to_add| {
                log_message!("Adding {} to {} profiles.", &i_type.id, profile.profile.as_str());
                let concrete_type = Self::create_bean_definition_concrete(i_type.clone());
                if !beans_to_add.contains(&concrete_type) {
                    beans_to_add.push(concrete_type)
                }
            });
    }

    fn contains_bean(bean_def_type: &BeanDefinitionType, to_check: &Vec<BeanDefinitionType>) -> bool {
        to_check.iter().any(|a| {
            match a {
                BeanDefinitionType::Abstract { bean, dep_type } => {
                    match bean_def_type {
                        BeanDefinitionType::Abstract { bean: inner_bean, dep_type: inner_dep_type } => {
                            bean.id == inner_bean.id
                                && (dep_type.item_impl.to_token_stream().to_string().as_str() == inner_dep_type.item_impl.to_token_stream().to_string().as_str()
                                    || dep_type.abstract_type == inner_dep_type.abstract_type)
                        }
                        BeanDefinitionType::Concrete { .. } => {
                            false
                        }
                    }
                }
                BeanDefinitionType::Concrete { bean } => {
                    false
                }
            }
        })
    }

    fn create_bean_definition_concrete(bean: BeanDefinition) -> BeanDefinitionType {
        info!("Creating concrete type for {}.", bean.id);
        BeanDefinitionType::Concrete {
            bean
        }
    }

    fn create_bean_definition_abstract(bean: BeanDefinition, dep_type: DependencyDescriptor) -> BeanDefinitionType {
        info!("Creating abstract type for {} and autowire type {}.",
            bean.id, dep_type.item_impl.to_token_stream().to_string().as_str());
        BeanDefinitionType::Abstract {
            bean, dep_type
        }
    }

    pub fn add_to_profile_abstract(&mut self, i_type: &BeanDefinition, profile: &ProfileBuilder, dep_type: DependencyDescriptor){
        self.injectable_types.get_mut(profile)
            .map(|beans_to_add| {
                    let bean_def_type = Self::create_bean_definition_abstract(i_type.clone(), dep_type);
                    if !beans_to_add.contains(&bean_def_type) {
                        beans_to_add.push(bean_def_type)
                    }
                }
            );
    }

    pub fn create_initial(beans: &HashMap<String, BeanDefinition>) -> HashMap<ProfileBuilder, Vec<BeanDefinitionType>> {

        let mut injectable_types = HashMap::new();

        injectable_types.insert(ProfileBuilder::default(), vec![]);

        beans.iter().flat_map(|bean| {
            let mut profiles = bean.1.profile.clone();
            bean.1.traits_impl.iter()
                .flat_map(|t| t.profile.clone())
                .for_each(|profile| profiles.push(profile));
            profiles
        }).for_each(|profile| {
            injectable_types.insert(profile, vec![]);
        });

        injectable_types
    }
}
