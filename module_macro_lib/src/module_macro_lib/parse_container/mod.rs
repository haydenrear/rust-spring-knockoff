use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::collections::hash_map::Keys;
use std::fmt::{Debug, Formatter};
use std::iter::Filter;
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::slice::Iter;
use std::str::pattern::Pattern;
use std::sync::{Arc};
use proc_macro2::TokenStream;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf, TypeArray, ItemEnum, ReturnType};
use syn::__private::str;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    LitStr,
    Token,
    Ident,
    token::Paren,
};
use quote::{quote, format_ident, IdentFragment, ToTokens, quote_token, TokenStreamExt};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use crate::FieldAugmenterImpl;
use crate::module_macro_lib::bean_parser::{BeanDependencyParser, BeanParser};
use crate::module_macro_lib::context_builder::ContextBuilder;
use crate::module_macro_lib::fn_parser::FnParser;
use crate::module_macro_lib::initializer::Initializer;
use crate::module_macro_lib::module_parser::parse_item;
use crate::module_macro_lib::module_tree::{Bean, Trait, Profile, DepType, BeanType, BeanDefinition, AutowiredField, AutowireType, InjectableTypeKey, ModulesFunctions, FunctionType, BeanDefinitionType};
use crate::module_macro_lib::profile_tree::ProfileTree;
use crate::module_macro_lib::knockoff_context_builder::ApplicationContextGenerator;
use crate::module_macro_lib::util::ParseUtil;
use knockoff_logging::{initialize_log, use_logging, create_logger_expr};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;

#[derive(Default)]
pub struct ParseContainer {
    pub injectable_types_builder: HashMap<String, Bean>,
    pub injectable_types_map: ProfileTree,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<TypeId, ModulesFunctions>,
    pub profiles: Vec<Profile>,
    pub initializer: Initializer
}

impl ParseContainer {

    /**
    Generate the token stream from the created ModuleContainer tree.
     **/
    pub fn build_to_token_stream(&mut self) -> TokenStream {
        ContextBuilder::build_token_stream(self)
    }

    pub fn build_injectable(&mut self) {
        self.injectable_types_map = ProfileTree::new(&self.injectable_types_builder);
        log_message!("{:?} is the debugged tree.", &self.injectable_types_map);
        log_message!("{} is the number of injectable types.", &self.injectable_types_builder.len());
    }

    /**
    1. Make sure that there are no cyclic dependencies.
    2. Reorder so that the beans are added to the container in the correct order.
    **/
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

    /**
    Add the struct and the impl from the ItemImpl
     **/
    pub fn create_update_impl(&mut self, item_impl: &mut ItemImpl) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        &mut self.injectable_types_builder.get_mut(&id)
            .map(|struct_impl: &mut Bean| {
                struct_impl.traits_impl.push(AutowireType { item_impl: item_impl.clone(), profile: vec![] });
            })
            .or_else(|| {
                let mut impl_found = Bean {
                    struct_type: Some(item_impl.self_ty.deref().clone()),
                    struct_found: None,
                    traits_impl: vec![AutowireType { item_impl: item_impl.clone(), profile: vec![] }],
                    enum_found: None,
                    attr: vec![],
                    deps_map: vec![],
                    id: id.clone(),
                    profile: vec![],
                    ident: None,
                    fields: vec![],
                    bean_type: None
                };
                self.injectable_types_builder.insert(id.clone(), impl_found);
                None
            });

        self.set_deps_safe(id.as_str());
    }

    pub fn add_item_struct(&mut self, item_impl: &mut ItemStruct) -> Option<String> {
        log_message!("adding type with name {}", item_impl.ident.clone().to_token_stream().to_string());
        log_message!("adding type with name {}", item_impl.to_token_stream().to_string().clone());

        self.injectable_types_builder.get_mut(&item_impl.ident.to_string().clone())
            .map(|struct_impl: &mut Bean| {
                struct_impl.struct_found = Some(item_impl.clone());
                struct_impl.ident =  Some(item_impl.ident.clone());
                struct_impl.fields = vec![item_impl.fields.clone()];
                struct_impl.bean_type = BeanParser::get_bean_type(&item_impl.attrs, None, Some(item_impl.ident.clone()));
                struct_impl.id = item_impl.ident.clone().to_string();
            })
            .or_else(|| {
                let mut impl_found = Bean {
                    struct_type: None,
                    struct_found: Some(item_impl.clone()),
                    traits_impl: vec![],
                    enum_found: None,
                    attr: vec![],
                    deps_map: vec![],
                    id: item_impl.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(item_impl.ident.clone()),
                    fields: vec![item_impl.fields.clone()],
                    bean_type: BeanParser::get_bean_type(&item_impl.attrs, None, Some(item_impl.ident.clone()))
                };
                self.injectable_types_builder.insert(item_impl.ident.to_string().clone(), impl_found);
                None
            });

        self.set_deps_safe(item_impl.ident.to_string().as_str());
        Some(item_impl.ident.to_string().clone())

    }

    pub fn add_item_enum(&mut self, enum_to_add: &mut ItemEnum) {
        log_message!("adding type with name {}", enum_to_add.ident.clone().to_token_stream().to_string());
        &mut self.injectable_types_builder.get_mut(&enum_to_add.ident.to_string().clone())
            .map(|struct_impl: &mut Bean| {
                struct_impl.enum_found = Some(enum_to_add.clone());
            })
            .or_else(|| {
                let enum_fields = enum_to_add.variants.iter()
                    .map(|variant| variant.fields.clone())
                    .collect::<Vec<Fields>>();
                let mut impl_found = Bean {
                    struct_type: None,
                    struct_found: None,
                    traits_impl: vec![],
                    enum_found: Some(enum_to_add.clone()),
                    attr: vec![],
                    deps_map: vec![],
                    id: enum_to_add.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(enum_to_add.ident.clone()),
                    fields: enum_fields,
                    bean_type: BeanParser::get_bean_type(&enum_to_add.attrs, None, Some(enum_to_add.ident.clone()))
                };
                self.injectable_types_builder.insert(enum_to_add.ident.to_string().clone(), impl_found);
                None
            });

        self.set_deps_safe(enum_to_add.ident.to_string().as_str());
    }

    fn set_deps_safe(&mut self, id: &str) {
        let mut removed = self.injectable_types_builder.remove(id).unwrap();
        let deps_set = BeanDependencyParser::add_dependencies(removed, &self.injectable_types_builder, &self.fns);
        self.injectable_types_builder.insert(id.clone().parse().unwrap(), deps_set);
    }

    pub fn create_update_trait(&mut self, trait_found: &mut ItemTrait) {
        if !self.traits.contains_key(&trait_found.ident.to_string().clone()) {
            self.traits.insert(trait_found.ident.to_string().clone(), Trait::new(trait_found.clone()));
        } else {
            log_message!("Contained trait already!");
        }
    }

    pub fn add_fn_to_dep_types(&mut self, item_fn: &mut ItemFn) {
        FnParser::to_fn_type(item_fn.clone())
            .map(|fn_found| {
                self.fns.insert(item_fn.clone().type_id().clone(), ModulesFunctions{ fn_found: fn_found.clone() });
                for i_type in self.injectable_types_builder.iter_mut() {
                    for dep_type in i_type.1.deps_map.iter_mut() {
                        if dep_type.bean_type.is_none() {
                            match &fn_found {
                                FunctionType::Singleton(fn_type, qualifier, type_found) => {
                                    dep_type.bean_type = Some(
                                        BeanType::Singleton(
                                            BeanDefinition {
                                                qualifier: qualifier.clone(),
                                                bean_type_type: type_found.clone(),
                                                bean_type_ident: None
                                            },
                                            Some(fn_found.clone()))
                                    );
                                }
                                FunctionType::Prototype(fn_type, qualifier, type_found) => {
                                    dep_type.bean_type = Some(
                                        BeanType::Prototype(
                                        BeanDefinition {
                                            qualifier: qualifier.clone(),
                                            bean_type_type: type_found.clone(),
                                            bean_type_ident: None
                                        },
                                        Some(fn_found.clone())
                                    ));
                                }
                            };
                        }
                    }
                }
            });
    }


    pub fn get_autowired_field_dep(attrs: Vec<Attribute>, field: Field) -> Option<AutowiredField> {
        log_message!("Checking attributes for field {}.", field.to_token_stream().to_string().clone());
        attrs.iter().map(|attr| {
            log_message!("Checking attribute: {} for field.", attr.to_token_stream().to_string().clone());
            let mut autowired_field = AutowiredField{
                qualifier: None,
                lazy: false,
                field: field.clone(),
                type_of_field: field.ty.clone(),
            };
            ParseContainer::get_qualifier_from_autowired(attr.clone())
                .map(|autowired_value| {
                    autowired_field.qualifier = Some(autowired_value);
                });
            Some(autowired_field)
        }).next().unwrap_or_else(|| {
            log_message!("Could not create autowired field of type {}.", field.ty.to_token_stream().to_string().clone());
            None
        })
    }

    pub fn get_qualifier_from_autowired(autowired_attr: Attribute) -> Option<String> {
        if autowired_attr.path.to_token_stream().to_string().clone().contains("autowired") {
            return ParseUtil::strip_value(autowired_attr.path.to_token_stream().to_string().as_str(), vec!["#[singleton(", "#[prototype("]);
        }
        None
    }

    pub fn get_type_from_fn_type(fn_type: &FunctionType) -> Option<Type> {
        match fn_type {
            FunctionType::Singleton(_, _, ty) => {
                ty.clone()
            }
            FunctionType::Prototype(_, _, ty) => {
                ty.clone()
            }
        }
    }



}
