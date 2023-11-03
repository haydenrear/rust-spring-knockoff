use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use syn::{Item, Type};
use quote::ToTokens;
use crate::bean::BeanDefinition;
use crate::functions::{FunctionType, ModulesFunctions};
use crate::module_tree::Trait;
use crate::profile_tree::{ProfileBuilder, ProfileTree};

use knockoff_logging::*;
use std::sync::Mutex;
use crate::logger_lazy;
import_logger!("parse_container.rs");


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
        self.injectable_types_builder.iter().filter(|&s| s.1.struct_found.as_ref().is_none() && s.1.ident.as_ref().is_none())
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


/// ItemModifier runs as the ParseContainer is loaded with the beans. It is running at the same
/// time as the code in module_macro_lib.item_parser
pub trait ParseContainerItemUpdater {
    fn parse_update(items: &mut Item, parse_container: &mut ParseContainer);
}

/// After the
/// 1. ParseContainerItemUpdater and the
/// 2. ItemModifier run
/// the final build is done, and
/// so the
/// 3. ParseContainerModifier is passed here to perform any finalizing changes.
/// 4. ProfileTreeFinalizer and Delegator for that
/// 5. TokenStreamGenerator and UserProvidedTokenStreamGenerator which calls the DelegatingTokenProvider
/// built with the CLI.
pub trait ParseContainerModifier {
    fn do_modify(items: &mut ParseContainer);
}

/// After the ItemModifier and the ParseContainerItemUpdater run, the final build is done, and
/// so the ParseContainer is passed here to perform any finalizing changes. This calls the
/// ParseContainerItemUpdater to build the parse container. After build parse container is called,
/// then the profile tree finalizer is called. Then after that, the token stream is generated
/// so then finally BuildProfileTree is called. This is when the TokenStream will be created.
pub trait BuildParseContainer {
    fn build_parse_container(&self, parse_container: &mut ParseContainer);
}
