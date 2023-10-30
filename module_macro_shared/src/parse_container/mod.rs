use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::sync::Arc;
use proc_macro2::TokenStream;
use syn::{Field, Type};
use codegen_utils::syn_helper::SynHelper;
use quote::ToTokens;
use crate::bean::BeanDefinition;
use crate::dependency::{AutowiredField, FieldDepType};
use crate::functions::{FunctionType, ModulesFunctions};
use crate::module_tree::Trait;
use crate::profile_tree::{ProfileBuilder, ProfileTree};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("parse_container.rs");
use crate::parse_container::parse_container_builder::BuildParseContainer;

pub mod parse_container_builder;
pub mod parse_container_modifier;

pub trait MetadataItem: 'static + Debug {
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Ord, PartialEq, Hash, Eq, PartialOrd, Clone, Debug)]
pub struct MetadataItemId {
    pub item_id: String,
    pub metadata_item_type_id: String
}

impl MetadataItemId {
    pub fn new(item_id: String, metadata_item_type_id: String) -> Self {
        Self {
            item_id, metadata_item_type_id
        }
    }
}

#[derive(Default)]
pub struct ParseContainer {
    pub injectable_types_builder: HashMap<String, BeanDefinition>,
    pub profile_tree: ProfileTree,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<String, ModulesFunctions>,
    pub profiles: Vec<ProfileBuilder>,
    pub provided_items: HashMap<MetadataItemId, Vec<Box<dyn MetadataItem>>>
}

impl Debug for ParseContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        log_message!("hello");
        Ok(())
    }
}

impl ParseContainer {

    pub fn get_injectable_keys(&self) -> Vec<String> {
        self.injectable_types_builder.keys().map(|k| k.clone()).collect()
    }

    pub fn log_app_container_info(&self) {
        self.injectable_types_builder.iter().filter(|&s| s.1.struct_found.is_none())
            .for_each(|s| {
                log_message!("Could not find struct type with ident {}.", s.0.clone());
            })
    }


    pub fn get_type_from_fn_type(fn_type: &FunctionType) -> Option<Type> {
        fn_type.fn_type.as_ref()
            .map(|f| f.get_inner_type())
            .flatten()
    }



}
