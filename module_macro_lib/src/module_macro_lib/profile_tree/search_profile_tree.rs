use knockoff_logging::{error, info};
use module_macro_shared::bean::BeanDefinitionType;
use module_macro_shared::dependency::{DependencyDescriptor, DependencyMetadata, DepType};
use module_macro_shared::profile_tree::{ProfileBuilder, ProfileTree};
use std::collections::HashMap;
use quote::ToTokens;

use knockoff_logging::*;

use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("concrete_profile_tree_modifier.rs");

pub trait SearchProfileTree {
    fn find_dependency<'a>(&'a self, dep_metadata: &'a DependencyMetadata,
                           profile_trees: &'a Vec<ProfileBuilder>,
                           qualifiers: &'a Vec<&str>
    ) -> Option<&'a BeanDefinitionType>;

    fn retrieve_dependency_from_profile_tree<'a>(
        &'a self,
        profiles: &'a Vec<ProfileBuilder>,
        dep_metadata: &'a DependencyMetadata
    ) -> HashMap<&'a ProfileBuilder, &'a BeanDefinitionType>;

}

impl SearchProfileTree for ProfileTree {

    fn find_dependency<'a>(
        &'a self,
        dep_metadata: &'a DependencyMetadata,
        profile_trees: &'a Vec<ProfileBuilder>,
        qualifiers: &'a Vec<&str>
    ) -> Option<&'a BeanDefinitionType>{
        let deps = self.retrieve_dependency_from_profile_tree(profile_trees, &dep_metadata);
        if deps.len() == 0 {
            error!("Could not find BeanDefinitionType for {:?}", dep_metadata);
            None
        } else {
            info!("{} bean def types were found.", deps.len());
            let option = deps.into_iter()
                .filter(|(profile, bean_def_type)| {
                    let vec = &bean_def_type.dep_type().unwrap().qualifiers;
                    info!("{:?} are the qualifiers", &vec);
                    let is_bean_non_prototype = vec.iter().any(|f| {
                        qualifiers.iter().any(|q| {
                            let f_str = f.as_str();
                            *q == f_str
                        })
                    });
                    let has_default = bean_def_type.bean().has_default();
                    if is_bean_non_prototype && has_default {
                        true
                    } else {
                        info!("{} is bean non prototype and {} doesn't have any of {:?} ", is_bean_non_prototype, has_default, &qualifiers);
                        false
                    }
                })
                .map(|(p, b)| b)
                .collect::<Vec<_>>();
            if option.len() != 0 {
                option.into_iter().next()
            } else {
                None
            }
        }

    }

    fn retrieve_dependency_from_profile_tree<'a>(
        &'a self,
        profiles: &'a Vec<ProfileBuilder>,
        dep_metadata: &'a DependencyMetadata
    ) -> HashMap<&'a ProfileBuilder, &'a BeanDefinitionType> {
        let ids = get_metadata_ids(&dep_metadata);
        info!("{:?} are the ids", &ids);

        profiles.iter()
            .flat_map(|p|
                self.injectable_types.get(p)
                    .into_iter()
                    .flat_map(|i| i.into_iter())
                    .filter(|b| {
                        info!("Found abstract bean to filter dep.");
                        if let BeanDefinitionType::Abstract {dep_type, bean} = b {
                            if does_dep_type_match(&dep_metadata, dep_type, Some(ids.clone())) {
                                info!("Found abstract bean to filter dep.");
                                true
                            } else {
                                info!("Bean dep did not match.");
                                false
                            }
                        }  else {
                            info!("Was not abstract bean def.");
                            false
                        }
                    })
                    .map(|b| (p, b))
                    .collect::<Vec<_>>()
            )
            .collect::<HashMap<&ProfileBuilder, &BeanDefinitionType>>()
    }



}


fn get_metadata_ids(dependency_metadata: &DependencyMetadata) -> Vec<String> {
    let mut ids = vec![];
    dependency_metadata.qualifier().as_ref().map(|q| ids.push(q.clone()));
    ids.push(dependency_metadata.dep_type_field_type().to_token_stream().to_string().clone());
    ids.push(dependency_metadata.dep_type_identifier().clone());
    ids.push(dependency_metadata.dep_type_field_type().to_token_stream().to_string().clone());
    dependency_metadata.dep_type_concrete_type().as_ref().map(|q| ids.push(q.to_token_stream().to_string().clone()));
    ids
}

pub(crate) fn does_dep_type_match<'a>(
    dependency_metadata: &'a DependencyMetadata,
    dependency_descriptor: &'a DependencyDescriptor,
    metadata: Option<Vec<String>>
) -> bool {
    info!("Testing if {:?} matches {:?}", dependency_metadata, dependency_descriptor);
    let metadata_ids = metadata.or(Some(get_metadata_ids(dependency_metadata)))
        .unwrap();
    let qualifiers = dependency_descriptor.qualifiers.iter().any(|q| {
        metadata_ids.iter().any(|m| m == q)
    });
    if qualifiers {
        return true
    }
    let item_impl = dependency_descriptor.item_impl.as_ref().map(|i| {
        let q = i.self_ty.to_token_stream().to_string();
        matchers_any_metadata(&metadata_ids, q)
    });
    if item_impl.as_ref().is_some() && item_impl.unwrap() {
        return true;
    }
    dependency_descriptor.abstract_type.as_ref()
        .map(|f| f.path_segments.get(0))
        .flatten()
        .map(|b| b.ident().as_ref()
            .map(|f| matchers_any_metadata(&metadata_ids, f.to_token_stream().to_string()))
        ).is_some()
}

pub(crate) fn matchers_any_metadata(metadata_ids: &Vec<String>, q: String) -> bool {
    metadata_ids.iter().any(|m| m.as_str() == q.as_str())
}
