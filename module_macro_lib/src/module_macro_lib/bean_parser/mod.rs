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
use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::bean::{BeanDefinition, BeanPath, BeanPathParts, BeanType};
use module_macro_shared::dependency::{AutowiredField, AutowiredFnArg, DependencyDescriptor, DependencyMetadata, FieldDepType};
use module_macro_shared::functions::{FunctionType, ModulesFunctions};
use crate::module_macro_lib::item_parser::item_fn_parser::ItemFnParser;
use module_macro_shared::parse_container::ParseContainer;
use crate::module_macro_lib::util::ParseUtil;

pub mod bean_dependency_path_parser;

pub struct BeanDependencyParser;

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

/// Add the DepType to Bean after all Beans are added.
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
        mut bean: BeanDefinition,
        injectable_types_builder: &HashMap<String, BeanDefinition>,
        fns: &HashMap<String, ModulesFunctions>
    ) -> BeanDefinition {
        if bean.factory_fn.as_ref().is_none() {
            Self::add_field_deps(&mut bean, injectable_types_builder, fns);
        } else {
            Self::add_fn_arg_deps(&mut bean, injectable_types_builder);
        }
        bean
    }

    fn add_fn_arg_deps(
        bean: &mut BeanDefinition,
        injectable_types_builder: &HashMap<String, BeanDefinition>
    ) {
        // TODO: add deps from fn_args, if mutable then add mutable...
        for mut arg in &bean.factory_fn.as_mut().unwrap().fn_found.args {
            // arg.2.map(|bean_id| injectable_types_builder.get(&bean_id))
            //     .flatten()
            //     .map(|dep_bean| )
        }
    }

    /// When the autowire type is a factory_fn, then there is no ImplItem that has been parsed, so they
    /// must be added again.
    fn get_dep_impl(qualifier: String, injectable_types: &HashMap<String, BeanDefinition>) -> Option<DependencyDescriptor> {
        injectable_types.values()
            .map(|v| {
                v.deps_map.iter()
                    .filter(|d| {
                        d.maybe_qualifier()
                            .as_ref()
                            .map(|s| s == &qualifier)
                            .is_some()
                    }
                    )
                    .map(|d| {
                        d.type_path().as_ref()
                            .map(|bean_type_path| {
                                DependencyDescriptor {
                                    item_impl: None,
                                    profile: vec![],
                                    path_depth: vec![],
                                    qualifiers: d.maybe_qualifier()
                                        .as_ref()
                                        .map(|q| vec![q.to_string()])
                                        .or(Some(vec![]))
                                        .unwrap(),
                                    abstract_type: Some(bean_type_path.clone()),
                                }
                            })
                    })
                    .flatten()
            })
            .flatten().next()
    }

    fn add_field_deps(mut bean: &mut BeanDefinition, injectable_types_builder: &HashMap<String, BeanDefinition>, fns: &HashMap<String, ModulesFunctions>) {
        for fields in bean.fields.clone().iter() {
            match fields.clone() {
                Fields::Named(fields_named) => {
                    for mut field in fields_named.named.iter() {
                        Self::match_ty_add_dep(
                            &mut bean,
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
    }

    pub fn get_autowired_field_dep(field: Field) -> Option<AutowiredField> {
        let profile = SynHelper::get_attr_from_vec(&field.attrs, vec!["profile"]);
        let qualifier = SynHelper::get_attr_from_vec(&field.attrs, vec!["qualifier"]);
        SynHelper::get_attr_from_vec(&field.attrs, vec!["autowired"])
            .map(|autowired_value| {
                let is_mutable = ParseUtil::does_attr_exist(&field.attrs, vec!["mutable_bean", "mutable_field"]);
                Self::log_autowired_info(&field, is_mutable);
                AutowiredField {
                    //TODO: this should be a vec
                    qualifier: Some(autowired_value.clone()).or(qualifier.clone()),
                    //TODO: this should be a vec
                    lazy: false,
                    field: field.clone(),
                    type_of_field: field.ty.clone(),
                    concrete_type_of_field_bean_type: None,
                    mutable: is_mutable,
                }
            })
    }

    pub fn get_autowired_fn_arg_dep(fn_arg_info: (Ident, BeanPath, Option<String>, PatType)) -> Option<AutowiredFnArg> {
        let profile = SynHelper::get_attr_from_vec(&fn_arg_info.3.attrs, vec!["profile"]);
        SynHelper::get_attr_from_vec(&fn_arg_info.3.attrs, vec!["autowired"])
            .map(|autowired_value| {
                let is_mutable = ParseUtil::does_attr_exist(&fn_arg_info.3.attrs, vec!["mutable_bean", "mutable_field"]);
                Self::log_autowired_info(&fn_arg_info.3, is_mutable);
                AutowiredFnArg{
                    //TODO: this should be a vec
                    qualifier: fn_arg_info.2.or(Some(autowired_value)),
                    //TODO: this should be a vec
                    profile: profile.clone(),
                    lazy: false,
                    fn_arg_ident: fn_arg_info.0,
                    bean_type: fn_arg_info.1.clone(),
                    type_of_field: fn_arg_info.1.get_inner_type()
                        .or(Some(fn_arg_info.3.ty.deref().clone()))
                        .unwrap(),
                    concrete_type_of_field_bean_type: None,
                    mutable: is_mutable,
                    fn_arg: fn_arg_info.3,
                }
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
        field: Field,
        injectable_types_builder: &HashMap<String, BeanDefinition>,
        fns: &HashMap<String, ModulesFunctions>
    ) {
        let autowired = Self::get_autowired_field_dep(field.clone());
        match autowired {
            None => {
            }
            Some(autowired) => {
                log_message!("Found field with type {}.", autowired.field.ty.to_token_stream().to_string().clone());
                if autowired.field.ident.is_some() {
                    log_message!("Found field with ident {}.", autowired.field.ident.to_token_stream().to_string().clone());
                }
                match field.ty.clone() {
                    Type::Array(arr) => {
                        log_message!("found array type {}.", arr.to_token_stream().to_string().clone());
                        Self::add_type_dep(dep_impl, autowired, lifetime, Some(arr), injectable_types_builder, fns, None);
                    }
                    Type::Path(path) => {
                        log_message!("Adding {} to bean path.", path.to_token_stream().clone().to_string().as_str());
                        let type_path = BeanDependencyPathParser::parse_type_path(path);
                        Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns, Some(type_path));
                    }
                    Type::Reference(reference_found) => {
                        let ref_type = reference_found.elem.clone();
                        log_message!("{} is the ref type", ref_type.to_token_stream());
                        Self::add_type_dep(dep_impl, autowired, reference_found.lifetime, array_type, injectable_types_builder, fns, None);
                    }
                    other => {
                        log_message!("{} is the other type", other.to_token_stream().to_string().as_str());
                        Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns, None)
                    }
                };
            }
        };
    }

    pub fn add_type_dep(
        dep_impl: &mut BeanDefinition,
        bean_info: AutowiredField,
        lifetime: Option<Lifetime>,
        array_type: Option<TypeArray>,
        injectable_types_builder: &HashMap<String, BeanDefinition>,
        fns: &HashMap<String, ModulesFunctions>,
        bean_type_path: Option<BeanPath>
    )
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
            let item_fn = Self::get_modules_fn(&bean_info, fns);

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
    }

    fn get_modules_fn(bean_info: &AutowiredField, fns: &HashMap<String, ModulesFunctions>) -> Option<ModulesFunctions> {
        ItemFnParser::get_fn_for_qualifier(fns, &bean_info.qualifier, &Some(bean_info.type_of_field.clone()))
    }

    fn get_bean_type(bean_info: &AutowiredField, injectable_types_builder: &HashMap<String, BeanDefinition>, fns: &HashMap<String, ModulesFunctions>) -> Option<BeanType> {
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
                ).map(|f| f.fn_found.bean_type)
            });
        bean_type
    }
}