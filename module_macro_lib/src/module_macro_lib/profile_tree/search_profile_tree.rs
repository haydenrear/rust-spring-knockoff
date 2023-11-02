use knockoff_logging::{error, info};
use module_macro_shared::bean::BeanDefinitionType;
use module_macro_shared::dependency::{DependencyDescriptor, DependencyMetadata, DepType};
use module_macro_shared::profile_tree::{ProfileBuilder, ProfileTree};
use std::collections::HashMap;
use quote::ToTokens;

use knockoff_logging::*;

use lazy_static::lazy_static;
use std::sync::Mutex;
use syn::ItemImpl;
use codegen_utils::project_directory;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use crate::logger_lazy;
import_logger!("concrete_profile_tree_modifier.rs");

pub trait SearchProfileTree {
    fn search_profile_tree<'a>(
        &'a self,
        dep_metadata: &'a DependencyMetadata,
        profile_trees: Option<&'a Vec<&'a ProfileBuilder>>,
        qualifiers: Option<&'a Vec<&str>>,
        profile_tree_search_criteria: &'a dyn ProfileTreeSearchCriteria,
    ) -> Vec<&'a BeanDefinitionType>;

    fn search_profile_all_profile_trees<'a>(
        &'a self, dep_metadata: &'a DependencyMetadata,
        profile_tree_search_criteria: &'a dyn ProfileTreeSearchCriteria
    ) -> Vec<&'a BeanDefinitionType>;

    fn retrieve_dependency_from_profile_tree<'a>(
        &'a self,
        profiles: Option<&'a Vec<&'a ProfileBuilder>>,
        dep_metadata: &'a DependencyMetadata,
        profile_tree_search_criteria: &'a dyn ProfileTreeSearchCriteria
    ) -> HashMap<&'a ProfileBuilder, &'a BeanDefinitionType>;

}

impl SearchProfileTree for ProfileTree {
    fn search_profile_all_profile_trees<'a>(
        &'a self,
        dep_metadata: &'a DependencyMetadata,
        profile_tree_search_criteria: &'a dyn ProfileTreeSearchCriteria
    ) -> Vec<&'a BeanDefinitionType> {
        self.search_profile_tree(dep_metadata, None, None, profile_tree_search_criteria)
    }

    fn search_profile_tree<'a>(
        &'a self,
        dep_metadata: &'a DependencyMetadata,
        profile_trees: Option<&'a Vec<&'a ProfileBuilder>>,
        qualifiers: Option<&'a Vec<&str>>,
        profile_tree_search_criteria: &'a dyn ProfileTreeSearchCriteria
    ) -> Vec<&'a BeanDefinitionType>{
        let deps = self.retrieve_dependency_from_profile_tree(
            profile_trees, &dep_metadata, profile_tree_search_criteria);
        if deps.len() == 0 {
            error!("Could not find BeanDefinitionType for {:?}", dep_metadata);
            vec![]
        } else {
            info!("{} bean def types were found.", deps.len());
            deps.into_iter()
                .filter(|(profile, bean_def_type)| {
                    let vec = &bean_def_type.dep_type().unwrap().qualifiers;
                    info!("{:?} are the qualifiers", &vec);
                    let is_bean_non_prototype = vec.iter().any(|f| {
                        qualifiers.map(|q| q.iter().any(|q| {
                            let f_str = f.as_str();
                            *q == f_str
                        })).or(Some(true)).unwrap()
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
                .collect::<Vec<_>>()
        }

    }

    fn retrieve_dependency_from_profile_tree<'a>(
        &'a self,
        profiles: Option<&'a Vec<&'a ProfileBuilder>>,
        dep_metadata: &'a DependencyMetadata,
        profile_tree_search_criteria: &'a dyn ProfileTreeSearchCriteria
    ) -> HashMap<&'a ProfileBuilder, &'a BeanDefinitionType> {
        let ids = get_metadata_ids(&dep_metadata);
        info!("{:?} are the ids", &ids);
        if profiles.is_none() {
            self.injectable_types
                .iter()
                .flat_map(|(p, t)| t
                    .iter()
                    .filter(|b| {
                        info!("Found abstract bean to filter dep.");
                        profile_tree_search_criteria.is_dependency_bean_def_type(&dep_metadata, &ids, b)
                    })
                    .map(move |b| (p, b))
                )
                .collect()
        } else {
            profiles
                .into_iter()
                .flat_map(|e| e.into_iter())
                .flat_map(|&p|
                    self.injectable_types.get(p)
                        .into_iter()
                        .flat_map(|i| i.into_iter())
                        .filter(|b| {
                            profile_tree_search_criteria.is_dependency_bean_def_type(&dep_metadata, &ids, b)
                        })
                        .map(|b| (p, b))
                        .collect::<Vec<_>>()
                )
                .collect::<HashMap<&ProfileBuilder, &BeanDefinitionType>>()
        }
    }

}

pub trait ProfileTreeSearchCriteria {
    fn is_dependency_bean_def_type(&self, dep_metadata: &&DependencyMetadata,
                                   ids: &Vec<String>,
                                   b: &BeanDefinitionType) -> bool {
        /// TODO:
        if let BeanDefinitionType::Abstract { dep_type, bean } = b {
            if does_dep_type_match(&dep_metadata, dep_type, Some(ids.clone())) {
                info!("Found abstract bean to filter dep.");
                true
            } else {
                info!("Bean dep did not match.");
                false
            }
        } else {
            info!("Was not abstract bean def.");
            false
        }
    }

    // fn does_self_ty_match(&self, bean_path: &str, item_impl_to_test: &ItemImpl) -> bool {
    //     let item_impl_self_ty = item_impl_to_test
    //         .self_ty.to_token_stream().to_string().as_str();
    //     if item_impl_self_ty == bean_path {
    //         return true;
    //     }
    //     false
    // }
}

#[derive(Default)]
pub struct IsAbstractProfileTreeSearchCriteria;

impl ProfileTreeSearchCriteria for IsAbstractProfileTreeSearchCriteria {}

// #[derive(Default)]
// pub struct MatchesDependencyTypeProvidedSearchCriteria;
//
// impl ProfileTreeSearchCriteria for MatchesDependencyTypeProvidedSearchCriteria {
//     fn is_dependency_bean_def_type(&self, dep_metadata: &&BeanDefinitionType, ids: &Vec<String>, b: &BeanDefinitionType) -> bool {
//         true
//     }
// }


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
