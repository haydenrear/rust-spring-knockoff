use std::any::{Any, TypeId};
use std::borrow::BorrowMut;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::collections::hash_map::Keys;
use std::fmt::{Debug, Formatter};
use std::iter::Filter;
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::slice::Iter;
use std::str::pattern::Pattern;
use std::sync::Arc;
use knockoff_providers_gen::{DelegatingProfileTreeFinalizerProvider, DelegatingProfileTreeModifierProvider};
use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Block, Data, DeriveInput, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, FnArg, ImplItem, ImplItemMethod, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Pat, Path, PatType, QSelf, ReturnType, Stmt, TraitItem, Type, TypeArray, TypePath};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    Ident,
    LitStr,
    Token,
    token::Paren,
};
use quote::{format_ident, IdentFragment, quote, quote_spanned, quote_token, TokenStreamExt, ToTokens};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use codegen_utils::syn_helper::SynHelper;
use crate::FieldAugmenterImpl;
use crate::module_macro_lib::bean_parser::BeanDependencyParser;
use crate::module_macro_lib::context_builder::ContextBuilder;
use crate::module_macro_lib::profile_tree::ProfileTreeBuilder;
use crate::module_macro_lib::knockoff_context_builder::ApplicationContextGenerator;
use crate::module_macro_lib::util::ParseUtil;
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType, BeanType};
use module_macro_shared::functions::{FunctionType, ModulesFunctions};
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
use module_macro_shared::module_tree::Trait;
use module_macro_shared::parse_container::parse_container_builder::BuildParseContainer;
use module_macro_shared::parse_container::ParseContainer;
use module_macro_shared::profile_tree::profile_tree_finalizer::ProfileTreeFinalizer;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use crate::module_macro_lib::profile_tree::concrete_profile_tree_modifier::ConcreteTypeProfileTreeModifier;
use crate::module_macro_lib::profile_tree::mutable_profile_tree_modifier::MutableProfileTreeModifier;
use crate::module_macro_lib::profile_tree::profile_profile_tree_modifier::ProfileProfileTreeModifier;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use crate::module_macro_lib::parse_container::parse_container_dependencies::{BuildDependencyParseContainer, DelegateParseContainerModifier};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use module_macro_shared::dependency::DepType;
use crate::logger_lazy;
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
            Box::new(DelegatingProfileTreeModifierProvider::new(&parse_container.injectable_types_builder)) as Box<dyn ProfileTreeModifier>
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
