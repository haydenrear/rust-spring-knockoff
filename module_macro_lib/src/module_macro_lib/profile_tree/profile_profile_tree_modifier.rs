use std::collections::HashMap;
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;
use module_macro_shared::profile_tree::ProfileBuilder;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use quote::ToTokens;
use codegen_utils::project_directory;
use codegen_utils::syn_helper::SynHelper;
use crate::logger_lazy;
use crate::module_macro_lib::bean_parser::bean_dependency_path_parser::BeanDependencyPathParser;
use crate::module_macro_lib::knockoff_context_builder::get_concrete_type_as_ident;
import_logger!("profile_profile_tree_modifier.rs");

pub struct ProfileProfileTreeModifier {
    default_profile: ProfileBuilder
}

impl ProfileTreeModifier for ProfileProfileTreeModifier {
    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        self.add_concrete_to_profile(dep_type, profile_tree);
        if profile_tree.injectable_types.contains_key(&self.default_profile) {
            log_message!("{} is the number of beans in the default profile after adding only the concrete beans.",
            profile_tree.injectable_types.get(&self.default_profile).unwrap().len());
        }
        self.add_abstract_to_profile(dep_type, profile_tree);
        if profile_tree.injectable_types.contains_key(&self.default_profile) {
            log_message!("{} is the number of beans in the default profile after adding the concrete and abstract beans.",
            profile_tree.injectable_types.get(&self.default_profile).unwrap().len());
        }
    }

    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self {
        Self {
            default_profile: Self::create_arg(profile_tree_items)
        }
    }
}

impl ProfileProfileTreeModifier {
    fn create_arg(profile_tree_items: &HashMap<String, BeanDefinition>) -> ProfileBuilder {
        ProfileBuilder::default()
    }
}

impl ProfileProfileTreeModifier {
    fn add_abstract_to_profile(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        let concrete_ty = get_concrete_type_as_ident(&dep_type.struct_type, &dep_type.ident);
        if concrete_ty.is_none() {
            error!("No concrete type for dep type: {} and bean definition {:?}.", &dep_type.id, dep_type);
        } else {
            dep_type.traits_impl.iter()
                .filter(|t| BeanDependencyPathParser::is_trait_abstract(&t.item_impl, &concrete_ty))
                .for_each(|trait_type| {
                    info!("Adding abstract dependency descriptor to profile: {:?}", trait_type);
                    BeanFactoryInfo::get_abstract_type(trait_type)
                        .as_ref()
                        .map(|t| {
                            log_message!("Creating abstract bean definition.");
                            profile_tree.add_to_profile_abstract(dep_type, &self.default_profile, trait_type.clone());
                            trait_type.profile
                                .iter()
                                .filter(|p| p.profile != self.default_profile.profile)
                                .for_each(|profile| {
                                    log_message!("Adding to profile {}", profile.profile.as_str());
                                    profile_tree.add_to_profile_abstract(dep_type, &profile, trait_type.clone());
                                });
                        });
                });
        }
    }


    fn add_concrete_to_profile(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        log_message!("Adding {} to default_impls.", dep_type.id.clone());
        profile_tree.add_to_profile_concrete(dep_type, &self.default_profile);
        dep_type.profile
            .iter()
            .filter(|p| p.profile != self.default_profile.profile)
            .for_each(|profile| {
                log_message!("Adding {} to profile {}.", dep_type.id.as_str(), profile.profile.as_str());
                profile_tree.add_to_profile_concrete(dep_type, profile);
            });
    }
}

