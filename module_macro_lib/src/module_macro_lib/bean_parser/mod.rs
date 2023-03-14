use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::Ident;
use quote::__private::ext::RepToTokensExt;
use quote::ToTokens;
use syn::{AngleBracketedGenericArguments, Attribute, Constraint, Field, Fields, GenericArgument, Lifetime, ParenthesizedGenericArguments, PathArguments, ReturnType, Type, TypeArray, TypeParamBound, TypePath};
use bean_dependency_path_parser::BeanDependencyPathParser;
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::bean::{Bean, BeanPath, BeanPathParts, BeanType};
use module_macro_shared::dependency::{AutowiredField, DepType};
use crate::module_macro_lib::item_parser::item_fn_parser::ItemFnParser;
use crate::module_macro_lib::parse_container::ParseContainer;
use crate::module_macro_lib::module_tree::{BeanDefinition, FunctionType, ModulesFunctions};
use crate::module_macro_lib::util::ParseUtil;

pub mod bean_dependency_path_parser;

pub struct BeanDependencyParser;

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

impl BeanDependencyParser {

    pub(crate) fn get_bean_type_opt(attr: &Vec<Attribute>) -> Option<BeanType> {
        SynHelper::get_attr_from_vec(attr, vec!["singleton"])
            .map(|singleton_qualifier| BeanType::Singleton)
            .or_else(|| {
                SynHelper::get_attr_from_vec(attr, vec!["prototype"])
                    .map(|singleton_qualifier| BeanType::Prototype)
            })
    }

    pub fn add_dependencies(
        mut bean: Bean,
        injectable_types_builder: &HashMap<String, Bean>,
        fns: &HashMap<TypeId, ModulesFunctions>
    ) -> Bean {
        for fields in bean.fields.clone().iter() {
            match fields.clone() {
                Fields::Named(fields_named) => {
                    for mut field in fields_named.named.iter() {
                        field.clone().ident.map(|ident: Ident| {
                            log_message!("found field {}.", ident.to_string().clone());
                        });
                        log_message!("{} is the field type!", field.ty.to_token_stream().clone());
                        bean = Self::match_ty_add_dep(
                            bean,
                            None,
                            None,
                            field.clone(),
                            injectable_types_builder,
                            fns
                        );
                    }
                }
                Fields::Unnamed(unnamed_field) => {}
                _ => {}
            };
        }
        bean
    }

    /**
    Adds the field to the to the tree as a dependency. Replace with DepImpl...
    **/
    pub fn match_ty_add_dep(
        mut dep_impl: Bean,
        lifetime: Option<Lifetime>,
        array_type: Option<TypeArray>,
        field: Field,
        injectable_types_builder: &HashMap<String, Bean>,
        fns: &HashMap<TypeId, ModulesFunctions>
    ) -> Bean {
        let autowired = ParseContainer::get_autowired_field_dep(field.clone());
        match autowired {
            None => {
                dep_impl
            }
            Some(autowired) => {
                log_message!("Found field with type {}.", autowired.field.ty.to_token_stream().to_string().clone());
                if autowired.field.ident.is_some() {
                    log_message!("Found field with ident {}.", autowired.field.ident.to_token_stream().to_string().clone());
                }
                match field.ty.clone() {
                    Type::Array(arr) => {
                        log_message!("found array type {}.", arr.to_token_stream().to_string().clone());
                        dep_impl = Self::add_type_dep(dep_impl, autowired, lifetime, Some(arr), injectable_types_builder, fns, None);
                    }
                    Type::Path(path) => {
                        log_message!("Adding {} to bean path.", path.to_token_stream().clone().to_string().as_str());
                        let type_path = BeanDependencyPathParser::parse_type_path(path);
                        dep_impl = Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns, Some(type_path));
                    }
                    Type::Reference(reference_found) => {
                        let ref_type = reference_found.elem.clone();
                        log_message!("{} is the ref type", ref_type.to_token_stream());
                        dep_impl = Self::add_type_dep(dep_impl, autowired, reference_found.lifetime, array_type, injectable_types_builder, fns, None);
                    }
                    other => {
                        log_message!("{} is the other type", other.to_token_stream().to_string().as_str());
                        dep_impl = Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns, None)
                    }
                };
                dep_impl
            }
        }
    }

    pub fn add_type_dep(
        mut dep_impl: Bean,
        bean_info: AutowiredField,
        lifetime: Option<Lifetime>,
        array_type: Option<TypeArray>,
        injectable_types_builder: &HashMap<String, Bean>,
        fns: &HashMap<TypeId, ModulesFunctions>,
        bean_type_path: Option<BeanPath>
    ) -> Bean
    {
        log_message!("Adding dependency for {}.", dep_impl.id.clone());

        let autowired_qualifier = bean_info.clone()
            .qualifier
            .or(Some(bean_info.type_of_field.to_token_stream().to_string().clone()));

        if autowired_qualifier.is_some() {

            log_message!("Adding dependency {} for bean with id {}.",  SynHelper::get_str(bean_info.field.clone()), dep_impl.id.clone());

            dep_impl.ident.clone().map(|ident| {
                log_message!("Adding dependency to struct with id {} to struct_impl of name {}", ident.to_string().clone(), SynHelper::get_str(&bean_info.type_of_field));
            }).or_else(|| {
                log_message!("Could not find ident for {} when attempting to add dependency to struct.", dep_impl.id.clone());
                None
            });
            bean_type_path.as_ref().map(|ident| {
                log_message!("Checking if has inner...");
                ident.get_inner_type().as_ref().map(|inner| {
                    log_message!("Adding dependency to struct with id {} to struct_impl with inner type {}", &dep_impl.id, SynHelper::get_str(inner));
                })
            }).or_else(|| {
                log_message!("Could not find inner type for dependency for {}.", dep_impl.id.clone());
                None
            });

            let bean_type = Self::get_bean_type(&bean_info, injectable_types_builder, fns);

            if bean_type.is_some() {
                dep_impl
                    .deps_map
                    .push(DepType {
                        bean_info,
                        lifetime,
                        bean_type,
                        array_type,
                        bean_type_path,
                        is_abstract: None,
                    });
            } else {
                dep_impl
                    .deps_map
                    .push(DepType {
                        bean_info,
                        lifetime,
                        bean_type: None,
                        array_type,
                        bean_type_path,
                        is_abstract: None,
                    });
            }
        }

        dep_impl
    }

    fn get_bean_type(bean_info: &AutowiredField, injectable_types_builder: &HashMap<String, Bean>, fns: &HashMap<TypeId, ModulesFunctions>) -> Option<BeanType> {
        let bean_type = bean_info.qualifier.as_ref()
            .map(|q| injectable_types_builder.get(q))
            .map(|b| b.map(|b| {
                b.bean_type.clone()
            }))
            .flatten()
            .flatten()
            .or_else(|| {
                ItemFnParser::get_fn_for_qualifier(
                    fns,
                    &bean_info.qualifier,
                    &Some(bean_info.type_of_field.clone())
                ).map(|f| f.bean_type)
            });
        bean_type
    }
}