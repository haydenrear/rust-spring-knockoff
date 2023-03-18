use std::cmp::Ordering;
use syn::{Attribute, Block, Field, Fields, ImplItemMethod, ItemEnum, ItemImpl, ItemStruct, Lifetime, Path, Stmt, Type, TypeArray};
use quote::ToTokens;
use std::fmt::{Debug, Formatter};
use std::fmt;
use codegen_utils::syn_helper;
use syn::__private::str;
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use proc_macro2::Ident;
use codegen_utils::syn_helper::SynHelper;

use std::collections::HashMap;
use crate::bean::{Bean, BeanDefinitionType};
use crate::dependency::{AutowireType, DepType};
use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::logging::executor;
use crate::logging::StandardLoggingFacade;

pub mod profile_tree_modifier;

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

#[derive(Clone, Default, Debug)]
pub struct ProfileTree {
    /// for profile implementations.
    pub injectable_types: HashMap<ProfileBuilder, Vec<BeanDefinitionType>>,
}

impl ProfileTree {

    pub fn add_to_profile_concrete(&mut self, i_type: &Bean, profile: &ProfileBuilder) {
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

    pub fn add_to_profile_abstract(&mut self, i_type: &Bean, profile: &ProfileBuilder, dep_type: AutowireType){
        self.injectable_types.get_mut(profile)
            .map(|beans_to_add| {
                    let bean_def_type = Self::create_bean_definition_abstract(i_type.clone(), dep_type);
                    if !beans_to_add.contains(&bean_def_type) {
                        beans_to_add.push(bean_def_type)
                    }
                }
            );
    }

    pub fn create_initial(beans: &HashMap<String,Bean>) -> HashMap<ProfileBuilder, Vec<BeanDefinitionType>> {

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
