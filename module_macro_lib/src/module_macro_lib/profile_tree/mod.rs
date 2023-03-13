use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::Meta::Path;
use syn::{parse2, Type};
use codegen_utils::syn_helper::SynHelper;
use concrete_profile_tree_modifier::ConcreteTypeProfileTreeModifier;
use crate::module_macro_lib::module_tree::{AutowireType, Bean, BeanDefinitionType, BeanPath, BeanPathParts, BeanType, DepType, InjectableTypeKey, Profile};

use knockoff_logging::{initialize_log, use_logging};
use mutable_profile_tree_modifier::MutableProfileTreeModifier;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::profile_tree::profile_profile_tree_modifier::ProfileProfileTreeModifier;
use crate::module_macro_lib::profile_tree::profile_tree_modifier::ProfileTreeModifier;

pub mod profile_tree_modifier;
pub mod mutable_profile_tree_modifier;
pub mod concrete_profile_tree_modifier;
pub mod profile_profile_tree_modifier;

#[derive(Clone, Default, Debug)]
pub struct ProfileTree {
    /// for profile implementations.
    pub injectable_types: HashMap<Profile, Vec<BeanDefinitionType>>,
}


impl ProfileTree {

    pub fn new(beans: &mut HashMap<String, Bean>) -> Self {

        let mut injectable_types = Self::create_initial(&beans);

        let mut profile_tree = Self {
            injectable_types,
        };

        let default_profile = Profile::default();

        let concrete_type_modifier = ConcreteTypeProfileTreeModifier::new(&beans);
        let mutable_type_modifier = MutableProfileTreeModifier::new(&beans);
        let profile_profile_tree_modifier = ProfileProfileTreeModifier::new(&beans);

        log_message!("{} is the number of beans parsed in profile tree.", beans.len());

        for mut i_type in beans.iter_mut() {

            concrete_type_modifier.modify_bean(i_type.1, &mut profile_tree);
            mutable_type_modifier.modify_bean(i_type.1, &mut profile_tree);
            profile_profile_tree_modifier.modify_bean(i_type.1, &mut profile_tree);

            log_message!("Adding {} to type.", i_type.1.id.clone());

        }

        log_message!("{:?} is the debugged profile tree.", &profile_tree);
        log_message!("{} is the number after.", profile_tree.injectable_types.get(&default_profile).unwrap().len());

        profile_tree
    }

    fn add_to_profile_concrete(&mut self, i_type: &Bean, profile: &Profile) {
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

    fn create_bean_definition_concrete(bean: Bean) -> BeanDefinitionType {
        print!("Creating concrete type for {}.", bean.id);
        BeanDefinitionType::Concrete {
            bean
        }
    }

    fn create_bean_definition_abstract(bean: Bean, dep_type: AutowireType) -> BeanDefinitionType {
        print!("Creating concrete type for {} and autowire type {}.", bean.id, dep_type.item_impl.to_token_stream().to_string().as_str());
        BeanDefinitionType::Abstract {
            bean, dep_type
        }
    }

    fn add_to_profile_abstract(&mut self, i_type: &Bean, profile: &Profile, dep_type: AutowireType){
        self.injectable_types.get_mut(profile)
            .map(|beans_to_add| {
                    let bean_def_type = Self::create_bean_definition_abstract(i_type.clone(), dep_type);
                    if !beans_to_add.contains(&bean_def_type) {
                        beans_to_add.push(bean_def_type)
                    }
                }
            );
    }

    fn create_initial(beans: &HashMap<String,Bean>) -> HashMap<Profile, Vec<BeanDefinitionType>> {

        let mut injectable_types = HashMap::new();

        injectable_types.insert(Profile::default(), vec![]);

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

