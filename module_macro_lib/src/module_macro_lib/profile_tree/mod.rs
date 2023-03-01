use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use proc_macro2::Ident;
use quote::ToTokens;
use crate::module_macro_lib::module_tree::{AutowireType, Bean, BeanDefinitionType, DepType, InjectableTypeKey, Profile};

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

#[derive(Clone, Default)]
pub struct ProfileTree {
    pub injectable_types: HashMap<Profile, Vec<BeanDefinitionType>>,
}

impl ProfileTree {

    pub fn new(beans: &mut HashMap<String, Bean>) -> Self {

        let mut injectable_types = Self::create_initial(&beans);

        let mut profile_tree = Self {
            injectable_types
        };

        let default_profile = Profile::default();

        let mutable_field_types: Vec<String> = beans.values()
            .filter(|b| b.mutable)
            .map(|b| b.ident.to_token_stream().to_string().clone())
            .collect::<Vec<String>>();


        for mut i_type in beans.iter_mut() {

            if mutable_field_types.iter().any(|i| i == i_type.0) {
                i_type.1.mutable = true;
            }

            log_message!("Adding {} to type.", i_type.1.id.clone());

            if i_type.1.profile.len() == 0 {
                log_message!("Adding {} to default_impls.", i_type.1.id.clone());
                profile_tree.add_to_profile_concrete(i_type.1, &default_profile);
            }
            i_type.1.profile.iter().for_each(|profile| {
                profile_tree.add_to_profile_concrete(i_type.1, profile);
            });
            i_type.1.traits_impl.iter()
                .for_each(|trait_type| {
                    if trait_type.profile.len() == 0 {
                        profile_tree.add_to_profile_abstract(i_type.1, &default_profile, trait_type.clone());
                    }
                    trait_type.profile
                        .iter()
                        .for_each(|profile| {
                            profile_tree.add_to_profile_abstract(i_type.1, &profile, trait_type.clone());
                        })
                });
        }

        log_message!("{:?} is the debugged profile tree.", &profile_tree);

        profile_tree
    }

    fn add_to_profile_concrete(&mut self, i_type: &Bean, profile: &Profile) {
        log_message!("Adding {} to {} profiles.", &i_type.id, profile.profile.as_str());
        self.injectable_types.get_mut(profile)
            .map(|beans_to_add| {
                log_message!("Adding {} to {} profiles.", &i_type.id, profile.profile.as_str());
                beans_to_add.push(Self::create_bean_definition_concrete(i_type.clone()))
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
            .map(|beans_to_add|
                beans_to_add.push(Self::create_bean_definition_abstract(i_type.clone(), dep_type))
            );
    }

    fn create_initial(beans: &HashMap<String,Bean>) -> HashMap<Profile, Vec<BeanDefinitionType>> {

        let mut injectable_types = HashMap::new();

        injectable_types.insert(Profile::default(), vec![]);

        beans.iter().flat_map(|bean| {
            let mut profiles = bean.1.profile.clone();
            bean.1.traits_impl.iter().flat_map(|t| t.profile.clone())
                .for_each(|profile| profiles.push(profile));
            profiles
        }).for_each(|profile| {
            injectable_types.insert(profile, vec![]);
        });

        injectable_types
    }
}