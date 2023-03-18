use std::collections::HashMap;
use module_macro_codegen::aspect::AspectParser;
use std::fmt::{Debug, Formatter};
use proc_macro2::TokenStream;
use syn::{Field, Type};
use codegen_utils::syn_helper::SynHelper;
use quote::ToTokens;
use crate::bean::Bean;
use crate::dependency::{AutowiredField, DepType};
use crate::functions::{FunctionType, ModulesFunctions};
use crate::item_modifier::DelegatingItemModifier;
use crate::module_tree::Trait;
use crate::profile_tree::{ProfileBuilder, ProfileTree};

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::logging::executor;
use crate::logging::StandardLoggingFacade;


#[derive(Default)]
pub struct ParseContainer {
    pub injectable_types_builder: HashMap<String, Bean>,
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

    pub fn get_autowired_field_dep(field: Field) -> Option<AutowiredField> {
        let qualifier = SynHelper::get_attr_from_vec(&field.attrs, vec!["profile"]);
        let profile = SynHelper::get_attr_from_vec(&field.attrs, vec!["qualifier"]);
        SynHelper::get_attr_from_vec(&field.attrs, vec!["autowired"])
            .map(|autowired_field| {
                log_message!("Attempting to add autowired field for {}.", field.to_token_stream().to_string().as_str());
                SynHelper::get_attr_from_vec(&field.attrs, vec!["mutable_bean"])
                    .map(|mutable_field| {
                        log_message!("Adding mutable field and autowired field for {}.", field.to_token_stream().to_string().as_str());
                        AutowiredField{
                            qualifier: Some(autowired_field.clone()).or(qualifier.clone()),
                            profile: profile.clone(),
                            lazy: false,
                            field: field.clone(),
                            type_of_field: field.ty.clone(),
                            concrete_type_of_field_bean_type: None,
                            mutable: true,
                        }
                    })
                    .or(Some(AutowiredField{
                        qualifier: Some(autowired_field).or(qualifier),
                        profile: profile.clone(),
                        lazy: false,
                        field: field.clone(),
                        type_of_field: field.ty.clone(),
                        concrete_type_of_field_bean_type: None,
                        mutable: false,
                    }))
            }).unwrap_or_else(|| {
                log_message!("Could not create autowired field of type {}.", field.ty.to_token_stream().to_string().clone());
                None
            })
    }

    pub fn get_type_from_fn_type(fn_type: &FunctionType) -> Option<Type> {
        fn_type.fn_type.as_ref()
            .map(|f| f.get_inner_type())
            .flatten()
    }



}
