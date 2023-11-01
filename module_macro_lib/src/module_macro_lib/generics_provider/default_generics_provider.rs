use std::collections::HashMap;
use quote::{quote, ToTokens};
use syn::{Field, Item, ItemMod, ItemStruct, Meta, MetaNameValue, NestedMeta, parse, parse2, Path, Type};
use knockoff_logging::{error, info};
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType, BeanPath, BeanType};
use module_macro_shared::dependency::{DependencyDescriptor, DependencyMetadata, DepType};
use module_macro_shared::item_modifier::ItemModifier;
use module_macro_shared::parse_container::ParseContainer;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::{ProfileBuilder, ProfileTree};
use crate::module_macro_lib::generics_provider::{GenericsProvider, GenericsResult, GenericsResultError};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use proc_macro2::{Ident, TokenStream};
use quote::__private::ext::RepToTokensExt;
use codegen_utils::project_directory;
use codegen_utils::syn_helper::SynHelper;
use crate::logger_lazy;
use crate::module_macro_lib::bean_parser::bean_dependency_path_parser::BeanDependencyPathParser;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::DefaultFieldInfo;
use crate::module_macro_lib::profile_tree::search_profile_tree::SearchProfileTree;
import_logger!("default_generics_provider.rs");

pub struct DefaultGenericsProvider;

impl ProfileTreeModifier for DefaultGenericsProvider {
    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        let _ = self.provide_generics(dep_type, profile_tree);
    }

    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self where Self: Sized {
        Self {}
    }
}


impl GenericsProvider for DefaultGenericsProvider {
    fn provide_generics(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) -> Result<GenericsResult, GenericsResultError>{
        info!("Running generics provider with {} deps map.", dep_type.deps_map.len());
        let _ = dep_type.deps_map.iter().for_each(|e| {
            info!("Running default generics provider.");
            let _ = Self::retrieve_generic_ty(e, dep_type, profile_tree,
                                              &vec![ProfileBuilder::default()]);
        });
        info!("{:?} are the qualifiers", &dep_type.qualifiers);
        if dep_type.qualifiers.iter().any(|q| q.as_str() == "default") && dep_type.has_default() {
            info!("Found contained default: {:?}", &dep_type.id);
        }
        Err(GenericsResultError {
            message: "".to_string(),
        })
    }
}

impl DefaultGenericsProvider {

    pub fn retrieve_generic_ty(
        dependency_metadata: &DependencyMetadata,
        bean_definition: &BeanDefinition,
        profile_tree: &mut ProfileTree,
        profile_trees: &Vec<ProfileBuilder>,
    ) -> Result<DefaultFieldInfo, GenericsResultError> {
        let qualifiers_to_search = vec!["default"];
        let default_field = profile_tree.find_dependency(
            dependency_metadata, &profile_trees, &qualifiers_to_search);
        if let Some(b)  = default_field {
            info!("Found default field: {:?}.", b);
            assert!(b.dep_type().as_ref().is_some(), "Bean definition type was not abstract!");
            let path = dependency_metadata.bean_info().bean_type();
            assert!(path.is_some(), "Abstract bean definition did not have path!");
            let path = path.unwrap();
            info!("Writing bean path for {:?}", path);
            Self::write_default_ty(path)
                .map(|path| {
                    DefaultFieldInfo {
                        field_ident: dependency_metadata.bean_info().field().unwrap().ident.clone().unwrap(),
                        field_type: path
                    }
                })
        } else {
            Err(GenericsResultError {
                message: "".to_string(),
            })
        }
    }

    pub fn write_default_ty(path: &BeanPath) -> Result<Type, GenericsResultError> {
        let head_value = path.path_segments[0].clone();
        let head_ty = head_value.gen_type().cloned();
        if let Some(ty) = head_ty  {
            info!("Writing default ty: {:?}", SynHelper::get_str(&ty));
            Ok(ty)
        } else {
            error!("Head value did not exist!");
            Err(GenericsResultError { message: "".to_string() })
        }
    }



}