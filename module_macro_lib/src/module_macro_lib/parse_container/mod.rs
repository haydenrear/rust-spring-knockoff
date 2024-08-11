use std::any::Any;
use std::borrow::BorrowMut;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Mutex;

use proc_macro2::TokenStream;
use quote::{IdentFragment, TokenStreamExt, ToTokens};
use serde::{Deserialize, Serialize};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::token::Token;

use knockoff_logging::*;
use knockoff_providers_gen::{DelegatingProfileTreeFinalizerProvider, DelegatingProfileTreeModifierProvider};
use module_macro_shared::{ProfileProfileTreeModifier, ProfileTreeBuilder};
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::dependency::DepType;
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
use module_macro_shared::parse_container::BuildParseContainer;
use module_macro_shared::parse_container::ParseContainer;
use module_macro_shared::profile_tree::profile_tree_finalizer::ProfileTreeFinalizer;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;

use crate::logger_lazy;
use crate::module_macro_lib::context_builder::ContextBuilder;
use crate::module_macro_lib::generics_provider::DelegatingGenericsProvider;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use crate::module_macro_lib::parse_container::parse_container_dependencies::{BuildDependencyParseContainer, DelegateParseContainerModifier};
use crate::module_macro_lib::profile_tree::concrete_profile_tree_modifier::ConcreteTypeProfileTreeModifier;
use crate::module_macro_lib::profile_tree::mutable_profile_tree_modifier::MutableProfileTreeModifier;

import_logger!("parse_container.rs");

pub mod parse_container_dependencies;

pub struct ParseContainerBuilder {
    parse_container_builders: Vec<Box<dyn BuildParseContainer>>
}

impl BuildParseContainer for ParseContainerBuilder {
    fn build_parse_container(&self, parse_container: &mut ParseContainer) {
        self.parse_container_builders.iter()
            .for_each(|p| p.build_parse_container(parse_container));
        Self::build_injectable(parse_container);
    }
}

impl ParseContainerBuilder {

    pub fn build_parse_container(parse_container: &mut ParseContainer) {
        ParseContainerBuilder::new().build(parse_container);
    }

    pub fn new() -> Self {
        Self {
            parse_container_builders: vec![
                Box::new(BuildDependencyParseContainer {}),
                Box::new(DelegateParseContainerModifier {})
            ]
        }
    }

    pub fn build_to_token_stream(parse_container: &mut ParseContainer) -> TokenStream {
        ContextBuilder::build_token_stream(parse_container)
    }

    pub fn build(&self, parse_container: &mut ParseContainer) {
        self.build_parse_container(parse_container);
    }

    pub fn build_injectable(parse_container: &mut ParseContainer) {

        let modifiers = vec![
            Box::new(ConcreteTypeProfileTreeModifier::new(&parse_container.injectable_types_builder)) as Box<dyn ProfileTreeModifier>,
            Box::new(MutableProfileTreeModifier::new(&parse_container.injectable_types_builder)) as Box<dyn ProfileTreeModifier>,
            Box::new(ProfileProfileTreeModifier::new(&parse_container.injectable_types_builder)) as Box<dyn ProfileTreeModifier>,
            Box::new(DelegatingProfileTreeModifierProvider::new(&parse_container.injectable_types_builder)) as Box<dyn ProfileTreeModifier>,
            Box::new(DelegatingGenericsProvider::new(&parse_container.injectable_types_builder)) as Box<dyn ProfileTreeModifier>
        ];

        parse_container.profile_tree = ProfileTreeBuilder::build_profile_tree(
            &mut parse_container.injectable_types_builder,
            modifiers,
            &mut parse_container.provided_items
        );

        DelegatingProfileTreeFinalizerProvider::finalize(parse_container);

        log_message!("{} is the number of injectable types in the profile tree.", &parse_container.profile_tree.injectable_types.values().len());
        log_message!("{:?} is the parsed and modified profile tree.", &parse_container.profile_tree);
    }


    pub fn is_valid_ordering_create(parse_container: &ParseContainer) -> Vec<String> {
        let mut already_processed = vec![];
        for i_type in parse_container.injectable_types_builder.iter() {
            if !Self::is_valid_ordering(parse_container, &mut already_processed, i_type.1) {
                log_message!("Was not valid ordering!");
                return vec![];
            }
        }
        already_processed
    }

    pub fn is_valid_ordering(parse_container: &ParseContainer, already_processed: &mut Vec<String>, dep: &BeanDefinition) -> bool {
        already_processed.push(dep.id.clone());
        for dep_impl in &dep.deps_map {
            let next_id = dep_impl.dep_type_identifier();
            if already_processed.contains(&next_id) {
                continue;
            }
            if !parse_container.injectable_types_builder.get(&next_id)
                .map(|next| {
                    return Self::is_valid_ordering(parse_container, already_processed, next);
                })
                .or(Some(false))
                .unwrap() {
                return false;
            }
        }
        true
    }
}
