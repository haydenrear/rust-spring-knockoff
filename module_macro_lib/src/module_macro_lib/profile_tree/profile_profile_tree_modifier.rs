use std::collections::HashMap;
use module_macro_shared::bean::Bean;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::profile_tree::ProfileBuilder;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub struct ProfileProfileTreeModifier {
    default_profile: ProfileBuilder
}

impl ProfileTreeModifier for ProfileProfileTreeModifier {
    fn modify_bean(&self, dep_type: &mut Bean, profile_tree: &mut ProfileTree) {
        self.add_concrete_to_profile(dep_type, profile_tree);
        log_message!("{} is the number of beans in the default profile after adding only the concrete beans.",
            profile_tree.injectable_types.get(&self.default_profile).unwrap().len());
        self.add_abstract_to_profile(dep_type, profile_tree);
        log_message!("{} is the number of beans in the default profile after adding the concrete and abstract beans.",
            profile_tree.injectable_types.get(&self.default_profile).unwrap().len());
    }

    fn new(profile_tree_items: &HashMap<String, Bean>) -> Self {
        Self {
            default_profile: Self::create_arg(profile_tree_items)
        }
    }
}

impl ProfileProfileTreeModifier {
    fn create_arg(profile_tree_items: &HashMap<String, Bean>) -> ProfileBuilder {
        ProfileBuilder::default()
    }
}

impl ProfileProfileTreeModifier {
    fn add_abstract_to_profile(&self, dep_type: &mut Bean, profile_tree: &mut ProfileTree) {
        dep_type.traits_impl.iter()
            .for_each(|trait_type| {
                trait_type.item_impl.trait_.as_ref().map(|t| {
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

    fn add_concrete_to_profile(&self, dep_type: &mut Bean, profile_tree: &mut ProfileTree) {
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

