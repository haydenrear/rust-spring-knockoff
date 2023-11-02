use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{Item, WhereClause, WherePredicate};
use module_macro_shared::bean::{AbstractionLevel, BeanDefinition, BeanPathParts, BeanType};
use module_macro_shared::generics::GenericsResult;
use module_macro_shared::impl_parse_values;
use module_macro_shared::item_modifier::ItemModifier;
use module_macro_shared::parse_container::{MetadataItem, MetadataItemId, ParseContainer, ParseContainerItemUpdater, ParseContainerModifier};
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;

/// TODO: map the generics to dyn types, which map to concrete bean factories.

pub struct GenericsResultError {
    message: String
}


/// All of the beans need to be in the container before the Generics TokenStream is set for
/// each of the BeanDefinitionTypes.
pub trait GenericsProvider: ProfileTreeModifier {
    fn provide_generics(&self, dep_type: &mut BeanDefinition,
                        profile_tree: &mut ProfileTree) -> Result<GenericsResult, GenericsResultError>;

}

pub struct DelegatingGenericsProvider {
    profile_tree_modifiers: Vec<Box<dyn GenericsProvider>>
}

impl ProfileTreeModifier for DelegatingGenericsProvider {
    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        self.profile_tree_modifiers.iter()
            .for_each(|m| m.modify_bean(dep_type, profile_tree));
    }

    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self where Self: Sized {
        Self {
            profile_tree_modifiers: vec![]
        }
    }
}