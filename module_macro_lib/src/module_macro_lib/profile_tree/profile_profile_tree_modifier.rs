use std::collections::HashMap;
use crate::module_macro_lib::module_tree::{Bean, Profile};
use crate::module_macro_lib::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use crate::module_macro_lib::profile_tree::ProfileTree;
use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub struct ProfileProfileTreeModifier {
    default_profile: Profile
}

impl ProfileTreeModifier<Profile> for ProfileProfileTreeModifier {
    fn create_arg(profile_tree_items: &HashMap<String, Bean>) -> Profile {
        Profile::default()
    }

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
    fn add_abstract_to_profile(&self, dep_type: &mut Bean, profile_tree: &mut ProfileTree) {
        dep_type.traits_impl.iter()
            .for_each(|trait_type| {
                if trait_type.profile.len() == 0 {
                    log_message!("Creating abstract bean definition.");
                    profile_tree.add_to_profile_abstract(dep_type, &self.default_profile, trait_type.clone());
                } else {
                    trait_type.profile
                        .iter()
                        .filter(|p| p.profile != self.default_profile.profile)
                        .for_each(|profile| {
                            log_message!("Adding to profile {}", profile.profile.as_str());
                            profile_tree.add_to_profile_abstract(dep_type, &profile, trait_type.clone());
                        })
                }
            });
    }

    fn add_concrete_to_profile(&self, dep_type: &mut Bean, profile_tree: &mut ProfileTree) {
        if dep_type.profile.len() == 0 {
            log_message!("Adding {} to default_impls.", dep_type.id.clone());
            profile_tree.add_to_profile_concrete(dep_type, &self.default_profile);
        }

        dep_type.profile.iter()
            .filter(|p| p.profile != self.default_profile.profile)
            .for_each(|profile| {
                log_message!("Adding {} to profile {}.", dep_type.id.as_str(), profile.profile.as_str());
                profile_tree.add_to_profile_concrete(dep_type, profile);
            });
    }
}

