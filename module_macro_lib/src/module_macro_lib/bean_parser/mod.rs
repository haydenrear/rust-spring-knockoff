use std::any::TypeId;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use std::path::Path;
use proc_macro2::Ident;
use quote::__private::ext::RepToTokensExt;
use quote::ToTokens;
use syn::{AngleBracketedGenericArguments, Attribute, Constraint, Field, Fields, GenericArgument, Lifetime, ParenthesizedGenericArguments, PathArguments, PatType, ReturnType, Type, TypeArray, TypeParamBound, TypePath};
use bean_dependency_path_parser::BeanDependencyPathParser;
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::{BeanDefinition, BeanPath, BeanPathParts, BeanType};
use module_macro_shared::dependency::{ArgDepType, AutowiredField, AutowiredFnArg, AutowiredType, DependencyDescriptor, DependencyMetadata, FieldDepType};
use module_macro_shared::functions::{FunctionType, ModulesFunctions};
use crate::module_macro_lib::item_parser::item_fn_parser::ItemFnParser;
use module_macro_shared::parse_container::ParseContainer;
use crate::module_macro_lib::util::ParseUtil;

pub mod bean_dependency_path_parser;

pub struct BeanDependencyParser;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("bean_parser.rs");

/// Add the DepType to Bean after all Beans are added.
impl BeanDependencyParser {
    pub(crate) fn get_bean_type_opt(attr: &Vec<Attribute>) -> Option<BeanType> {
        SynHelper::get_attr_from_vec(attr, &vec!["service"])
            .map(|singleton_qualifier| BeanType::Singleton)
            .or_else(|| {
                SynHelper::get_attr_from_vec(attr, &vec!["prototype"])
                    .map(|singleton_qualifier| BeanType::Prototype)
            })
    }

    pub fn add_dependencies(
        mut bean: BeanDefinition,
        injectable_types_builder: &HashMap<String, BeanDefinition>,
        fns: &HashMap<String, ModulesFunctions>,
    ) -> BeanDefinition {
        if bean.factory_fn.as_ref().is_none() {
            Self::add_field_deps(&mut bean, injectable_types_builder, fns);
        } else {
            Self::add_fn_arg_deps(&mut bean, injectable_types_builder, fns);
        }
        bean
    }

    fn add_fn_arg_deps(
        mut bean: &mut BeanDefinition,
        injectable_types_builder: &HashMap<String, BeanDefinition>,
        fns: &HashMap<String, ModulesFunctions>
    ) {
        Self::add_field_deps(&mut bean, injectable_types_builder, fns)
    }

    fn add_field_deps(mut bean: &mut BeanDefinition,
                      injectable_types_builder: &HashMap<String, BeanDefinition>,
                      fns: &HashMap<String, ModulesFunctions>) {
        bean.factory_fn.as_ref()
            .map(|m| m.fn_found.args.clone())
            .map(|factory_fn| {
                Self::add_fn_arg_deps_to_bean(&mut bean, injectable_types_builder, fns, factory_fn);
            }).or_else(|| {
                Self::add_field_deps_to_bean(&mut bean, injectable_types_builder, fns)
            });
    }

    fn add_fn_arg_deps_to_bean(mut bean: &mut &mut BeanDefinition, injectable_types_builder: &HashMap<String, BeanDefinition>, fns: &HashMap<String, ModulesFunctions>, factory_fn: Vec<(Ident, BeanPath, Option<String>, PatType)>) {
        factory_fn.iter()
            .for_each(|data| {
                Self::match_ty_add_dep(
                    &mut bean,
                    None,
                    None,
                    injectable_types_builder,
                    fns,
                    Self::get_autowired_fn_arg_dep(data),
                )
            })
    }

    fn add_field_deps_to_bean(mut bean: &mut &mut BeanDefinition, injectable_types_builder: &HashMap<String, BeanDefinition>, fns: &HashMap<String, ModulesFunctions>) -> Option<()> {
        for fields in bean.fields.clone().iter() {
            match fields.clone() {
                Fields::Named(fields_named) => {
                    for mut field in fields_named.named.iter() {
                        Self::match_ty_add_dep(
                            &mut bean,
                            None,
                            None,
                            injectable_types_builder,
                            fns,
                            Self::get_autowired_field_dep(&field),
                        );
                    }
                }
                Fields::Unnamed(unnamed_field) => {}
                _ => {}
            };
        }
        None
    }

    pub fn get_autowired_field_dep(field: &Field) -> Option<AutowiredType> {
        let profile = SynHelper::get_attr_from_vec(&field.attrs, &vec!["profile"]);
        let qualifier = SynHelper::get_attr_from_vec(&field.attrs, &vec!["qualifier"]);
        SynHelper::get_attr_from_vec(&field.attrs, &vec!["autowired"])
            .map(|autowired_value| {
                let is_mutable = ParseUtil::does_attr_exist(&field.attrs, &vec!["mutable_bean", "mutable_field"]);
                Self::log_autowired_info(&field, is_mutable);
                AutowiredType::Field(AutowiredField {
                    //TODO: this should be a vec
                    qualifier: Some(autowired_value.clone()).or(qualifier.clone()),
                    //TODO: this should be a vec
                    lazy: false,
                    field: field.clone(),
                    autowired_type: field.ty.clone(),
                    concrete_type_of_field_bean_type: None,
                    mutable: is_mutable,
                })
            })
    }

    pub fn get_autowired_fn_arg_dep(fn_arg_info: &(Ident, BeanPath, Option<String>, PatType)) -> Option<AutowiredType> {
        let profile = SynHelper::get_attr_from_vec(&fn_arg_info.3.attrs, &vec!["profile"]);
        SynHelper::get_attr_from_vec(&fn_arg_info.3.attrs, &vec!["autowired"])
            .map(|autowired_value| {
                let is_mutable = ParseUtil::does_attr_exist(&fn_arg_info.3.attrs, &vec!["mutable_bean", "mutable_field"]);
                Self::log_autowired_info(&fn_arg_info.3, is_mutable);
                AutowiredType::FnArg(AutowiredFnArg {
                    //TODO: this should be a vec
                    qualifier: fn_arg_info.2.clone().or(Some(autowired_value)),
                    //TODO: this should be a vec
                    profile: profile.clone(),
                    lazy: false,
                    fn_arg_ident: fn_arg_info.0.clone(),
                    bean_type: fn_arg_info.1.clone(),
                    autowired_type: fn_arg_info.1.get_inner_type()
                        .or(Some(fn_arg_info.3.ty.deref().clone()))
                        .unwrap(),
                    concrete_type_of_field_bean_type: None,
                    mutable: is_mutable,
                    fn_arg: fn_arg_info.3.clone(),
                })
            })
    }

    fn log_autowired_info<T: ToTokens>(fn_arg_info: T, is_mutable: bool) {
        log_message!("Attempting to add {} autowired for {}.",
            Self::mutable_or_nonmutable(is_mutable),
            SynHelper::get_str(&fn_arg_info)
        );
    }

    fn mutable_or_nonmutable(is_mutable: bool) -> &'static str {
        if is_mutable {
            "mutable"
        } else {
            "non-mutable"
        }
    }

    /**
    Adds the field to the to the tree as a dependency. Replace with DepImpl...
    **/
    pub fn match_ty_add_dep(
        dep_impl: &mut BeanDefinition,
        lifetime: Option<Lifetime>,
        array_type: Option<TypeArray>,
        injectable_types_builder: &HashMap<String, BeanDefinition>,
        fns: &HashMap<String, ModulesFunctions>,
        autowired: Option<AutowiredType>,
    ) {
        autowired.map(|autowired| {
            log_message!("Found field with ident {}.", SynHelper::get_str(autowired.autowired_type()));
            match autowired.autowired_type().clone() {
                Type::Array(arr) => {
                    log_message!("found array type {}.", arr.to_token_stream().to_string().clone());
                    Self::add_type_dep(dep_impl, autowired, lifetime, Some(arr.clone()), injectable_types_builder, fns, None);
                }
                Type::Path(path) => {
                    log_message!("Adding {} to bean path.", path.to_token_stream().clone().to_string().as_str());
                    let type_path = BeanDependencyPathParser::parse_type_path(path.clone());
                    Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns, Some(type_path));
                }
                Type::Reference(reference_found) => {
                    let ref_type = reference_found.elem.clone();
                    log_message!("{} is the ref type", ref_type.to_token_stream());
                    Self::add_type_dep(dep_impl, autowired, reference_found.clone().lifetime, array_type, injectable_types_builder, fns, None);
                }
                other => {
                    log_message!("{} is the other type", other.to_token_stream().to_string().as_str());
                    Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns, None)
                }
            };
        });
    }

    pub fn add_type_dep(
        dep_impl: &mut BeanDefinition,
        bean_info: AutowiredType,
        lifetime: Option<Lifetime>,
        array_type: Option<TypeArray>,
        injectable_types_builder: &HashMap<String, BeanDefinition>,
        fns: &HashMap<String, ModulesFunctions>,
        bean_type_path: Option<BeanPath>,
    )
    {
        log_message!("Adding dependency for {}.", dep_impl.id.clone());

        let autowired_qualifier = bean_info.qualifier();

        if autowired_qualifier.is_some() {
            let bean_type = Self::get_bean_type(&bean_info, injectable_types_builder, fns);

            match bean_info {
                AutowiredType::Field(bean_info) => {
                    if bean_type.is_some() {
                        dep_impl
                            .deps_map
                            .push(DependencyMetadata::FieldDepType(FieldDepType {
                                bean_info,
                                lifetime,
                                bean_type,
                                array_type,
                                bean_type_path,
                                is_abstract: None,
                            }));
                    } else {
                        dep_impl
                            .deps_map
                            .push(DependencyMetadata::FieldDepType(FieldDepType {
                                bean_info,
                                lifetime,
                                bean_type: None,
                                array_type,
                                bean_type_path,
                                is_abstract: None,
                            }));
                    }
                }

                AutowiredType::FnArg(bean_info) => {
                    if bean_type.is_some() {
                        dep_impl
                            .deps_map
                            .push(DependencyMetadata::ArgDepType(ArgDepType {
                                bean_info,
                                lifetime,
                                bean_type,
                                array_type,
                                bean_type_path,
                                is_abstract: None,
                            }));
                    } else {
                        dep_impl
                            .deps_map
                            .push(DependencyMetadata::ArgDepType(ArgDepType {
                                bean_info,
                                lifetime,
                                bean_type: None,
                                array_type,
                                bean_type_path,
                                is_abstract: None,
                            }));
                    }
                }
            }
        }
    }


    fn get_modules_fn(bean_info: &AutowiredType, fns: &HashMap<String, ModulesFunctions>) -> Option<ModulesFunctions> {
        ItemFnParser::get_fn_for_qualifier(fns, bean_info.qualifier(), &Some(bean_info.autowired_type()))
    }

    fn get_bean_type(bean_info: &AutowiredType, injectable_types_builder: &HashMap<String, BeanDefinition>, fns: &HashMap<String, ModulesFunctions>) -> Option<BeanType> {
        let bean_type = bean_info.qualifier().clone()
            .map(|q| injectable_types_builder.get(&q))
            .map(|b| b.map(|b| {
                b.bean_type.clone()
            }))
            .flatten()
            .flatten()
            .or_else(|| {
                ItemFnParser::get_fn_for_qualifier(
                    fns,
                    bean_info.qualifier(),
                    &Some(bean_info.autowired_type()),
                ).map(|f| f.fn_found.bean_type)
            });
        bean_type
    }
}