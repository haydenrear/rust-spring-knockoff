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
use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Block, Data, DeriveInput, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, FnArg, ImplItem, ImplItemMethod, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Pat, Path, PatType, QSelf, ReturnType, Stmt, TraitItem, Type, TypeArray, TypePath};
use syn::__private::{str, TokenStream as ts};
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
use crate::module_macro_lib::initializer::ModuleMacroInitializer;
use crate::module_macro_lib::module_tree::{BeanDefinition, FunctionType, InjectableTypeKey, ModulesFunctions, Trait};
use crate::module_macro_lib::profile_tree::ProfileTreeBuilder;
use crate::module_macro_lib::knockoff_context_builder::ApplicationContextGenerator;
use crate::module_macro_lib::util::ParseUtil;
use knockoff_logging::{create_logger_expr, initialize_log, use_logging};
use module_macro_codegen::aspect::{AspectParser, MethodAdviceAspectCodegen};
use module_macro_shared::aspect::AspectInfo;
use module_macro_shared::bean::{Bean, BeanDefinitionType, BeanType};
use module_macro_shared::dependency::{AutowiredField, AutowireType, DepType};
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
use module_macro_shared::profile_tree::{ProfileBuilder, ProfileTree};
use crate::module_macro_lib::item_modifier::delegating_modifier::DelegatingItemModifier;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::profile_tree::concrete_profile_tree_modifier::ConcreteTypeProfileTreeModifier;
use crate::module_macro_lib::profile_tree::mutable_profile_tree_modifier::MutableProfileTreeModifier;
use crate::module_macro_lib::profile_tree::profile_profile_tree_modifier::ProfileProfileTreeModifier;
use crate::module_macro_lib::profile_tree::profile_tree_modifier::ProfileTreeModifier;

#[derive(Default)]
pub struct ParseContainer {
    pub injectable_types_builder: HashMap<String, Bean>,
    pub profile_tree: ProfileTree,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<TypeId, ModulesFunctions>,
    pub profiles: Vec<ProfileBuilder>,
    pub initializer: ModuleMacroInitializer,
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

    pub fn build_to_token_stream(&mut self) -> TokenStream {
        ContextBuilder::build_token_stream(self)
    }

    pub fn build_injectable(&mut self) {
        self.set_build_dep_types();

        let modifiers = vec![
            Box::new(ConcreteTypeProfileTreeModifier::new(&self.injectable_types_builder)) as Box<dyn ProfileTreeModifier>,
            Box::new(MutableProfileTreeModifier::new(&self.injectable_types_builder)) as Box<dyn ProfileTreeModifier>,
            Box::new(ProfileProfileTreeModifier::new(&self.injectable_types_builder)) as Box<dyn ProfileTreeModifier>
        ];

        self.profile_tree = ProfileTreeBuilder::build_profile_tree(&mut self.injectable_types_builder, modifiers);

        log_message!("{} is the number of injectable types in the profile tree.", &self.profile_tree.injectable_types.values().len());
        log_message!("{:?} is the parsed and modified profile tree.", &self.profile_tree);

    }

    pub fn set_build_dep_types(&mut self) {
        let keys = self.get_injectable_keys();
        let fns = self.fns.values().map(|fn_found| fn_found.fn_found.clone())
            .collect::<Vec<FunctionType>>();
        log_message!("{} is the number of injectable keys before.", keys.len());
        for id in keys.iter() {
            let mut removed = self.injectable_types_builder.remove(id).unwrap();
            let deps_set = BeanDependencyParser::add_dependencies(removed, &self.injectable_types_builder, &self.fns);
            self.injectable_types_builder.insert(id.clone().parse().unwrap(), deps_set);
        }
        for fn_type in fns.iter() {
            self.set_fn_type_dep(&fn_type);
        }
        log_message!("{} is the number of injectable keys after.", self.injectable_types_builder.len());
    }

    pub fn get_injectable_keys(&self) -> Vec<String> {
        self.injectable_types_builder.keys().map(|k| k.clone()).collect()
    }

    pub fn is_valid_ordering_create(&self) -> Vec<String> {
        let mut already_processed = vec![];
        for i_type in self.injectable_types_builder.iter() {
            if !self.is_valid_ordering(&mut already_processed, i_type.1) {
                log_message!("Was not valid ordering!");
                return vec![];
            }
        }
        already_processed
    }

    pub fn is_valid_ordering(&self, already_processed: &mut Vec<String>, dep: &Bean) -> bool {
        already_processed.push(dep.id.clone());
        for dep_impl in &dep.deps_map {
            let next_id = ParseContainer::get_identifier(dep_impl);
            if already_processed.contains(&next_id) {
                continue;
            }
            if !self.injectable_types_builder.get(&next_id)
                .map(|next| {
                    return self.is_valid_ordering(already_processed, next);
                })
                .or(Some(false))
                .unwrap() {
                return false;
            }
        }
        true
    }

    pub fn get_identifier(dep_type: &DepType) -> String {
        match &dep_type.bean_info.qualifier  {
            None => {
                dep_type.bean_info.type_of_field.to_token_stream().to_string()
            }
            Some(qual) => {
                qual.clone()
            }
        }
    }

    pub fn log_app_container_info(&self) {
        self.injectable_types_builder.iter().filter(|&s| s.1.struct_found.is_none())
            .for_each(|s| {
                log_message!("Could not find struct type with ident {}.", s.0.clone());
            })
    }


    fn set_fn_type_dep(&mut self, fn_found: &FunctionType) {
        for i_type in self.injectable_types_builder.iter_mut() {
            for dep_type in i_type.1.deps_map.iter_mut() {
                if dep_type.bean_type.is_none() {
                    dep_type.bean_type = Some(fn_found.bean_type.clone());
                }
            }
        }
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
        fn_type.fn_type.clone()
    }



}
