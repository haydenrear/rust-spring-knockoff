use std::collections::HashMap;
use module_macro_codegen::aspect::AspectParser;
use std::fmt::{Debug, Formatter};
use proc_macro2::TokenStream;
use syn::{Field, Type};
use codegen_utils::syn_helper::SynHelper;
use quote::ToTokens;
use crate::bean::BeanDefinition;
use crate::dependency::{AutowiredField, FieldDepType};
use crate::functions::{FunctionType, ModulesFunctions};
use crate::item_modifier::DelegatingItemModifier;
use crate::module_tree::Trait;
use crate::profile_tree::{ProfileBuilder, ProfileTree};

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::logging::executor;
use crate::logging::StandardLoggingFacade;
use crate::parse_container::parse_container_builder::BuildParseContainer;

pub mod parse_container_builder;

#[derive(Default)]
pub struct ParseContainer {
    pub injectable_types_builder: HashMap<String, BeanDefinition>,
    pub profile_tree: ProfileTree,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<String, ModulesFunctions>,
    pub profiles: Vec<ProfileBuilder>,
    pub aspects: AspectParser,
    pub item_modifier: DelegatingItemModifier
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
