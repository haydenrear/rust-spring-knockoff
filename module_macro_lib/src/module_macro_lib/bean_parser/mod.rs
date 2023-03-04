use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::Ident;
use quote::__private::ext::RepToTokensExt;
use quote::ToTokens;
use syn::{AngleBracketedGenericArguments, Attribute, Constraint, Field, Fields, GenericArgument, Lifetime, ParenthesizedGenericArguments, PathArguments, ReturnType, Type, TypeArray, TypeParamBound, TypePath};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use crate::module_macro_lib::fn_parser::FnParser;
use crate::module_macro_lib::parse_container::ParseContainer;
use crate::module_macro_lib::module_tree::{AutowiredField, Bean, BeanDefinition, BeanPath, BeanPathParts, BeanType, DepType, FunctionType, ModulesFunctions};
use crate::module_macro_lib::util::ParseUtil;

pub struct BeanParser;
pub struct BeanDependencyParser;

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

impl BeanParser {
    pub(crate) fn get_prototype_or_singleton(attr: &Vec<Attribute>, bean_type: Option<Type>, bean_type_ident: Option<Ident>) -> Option<BeanType> {
        ParseUtil::filter_att(attr, vec!["singleton", "prototype"])
            .and_then(|s| {
                let qualifier = ParseUtil::strip_value_attr(s, vec!["#[singleton(", "#[prototype("]);

                qualifier.iter()
                    .for_each(|qual| {
                        log_message!("Found bean with qualifier {}.", qual);
                    });

                log_message!("Found bean with attr {}.", s.to_token_stream().to_string().as_str());
                if s.path.to_token_stream().to_string().as_str().contains("singleton") {
                    return Some(BeanType::Singleton(BeanDefinition{
                        qualifier: qualifier,
                        bean_type_type: bean_type,
                        bean_type_ident
                    }, None))
                        .map(|bean_type| {
                            log_message!("Found singleton bean: {:?}.", bean_type);
                            bean_type
                        })
                } else if s.path.to_token_stream().to_string().as_str().contains("prototype") {
                    return Some(BeanType::Prototype(BeanDefinition{
                        qualifier: qualifier,
                        bean_type_type: bean_type,
                        bean_type_ident
                    }, None))
                        .map(|bean_type| {
                            log_message!("Found singleton bean: {:?}.", bean_type);
                            bean_type
                        })
                }
                None
            })
    }


    pub fn get_bean_type_from_factory_fn(attrs: Vec<Attribute>, module_fn: ModulesFunctions) -> Option<BeanType> {
        if attrs.iter().any(|attr| {
            let attr_str = attr.to_token_stream().to_string();
            attr_str.contains("bean") || attr_str.contains("singleton") || attr_str.contains("prototype")
        }) {
            return attrs.iter().flat_map(|attr| {
                let qualifier = ParseUtil::strip_value(attr.path.to_token_stream().to_string().as_str(), vec!["#[singleton(", "#[prototype("]);
                if attr.to_token_stream().to_string().contains("singleton") {
                    return Some(
                        BeanType::Singleton(
                            BeanDefinition{
                                qualifier,
                                bean_type_type: ParseContainer::get_type_from_fn_type(&module_fn.fn_found),
                                bean_type_ident: None
                            },
                            Some(module_fn.fn_found.clone())
                        ));
                } else if attr.to_token_stream().to_string().contains("prototype") {
                    return Some(BeanType::Prototype(
                        BeanDefinition{
                            qualifier,
                            bean_type_type: ParseContainer::get_type_from_fn_type(&module_fn.fn_found),
                            bean_type_ident: None
                        },
                        Some(module_fn.fn_found.clone())
                    ));
                }
                None
            }).next()
        }
        None
    }

    pub fn get_bean_type_from_qual(qualifier: Option<String>, type_type: Option<Type>, module_fn: FunctionType) -> Option<BeanType> {
        match &module_fn {
            FunctionType::Singleton(_, qualifier_found, _) => {
                return Some(
                    BeanType::Singleton(
                        BeanDefinition{
                            qualifier: qualifier_found.clone(),
                            bean_type_type: ParseContainer::get_type_from_fn_type(&module_fn),
                            bean_type_ident: None
                        },
                        Some(module_fn)
                    ));
            }
            FunctionType::Prototype(_, qualifier_found, _) => {
                return Some(BeanType::Prototype(
                    BeanDefinition{
                        qualifier: qualifier_found.clone(),
                        bean_type_type: ParseContainer::get_type_from_fn_type(&module_fn),
                        bean_type_ident: None
                    },
                    Some(module_fn)
                ));
            }
        }
    }

    pub fn get_bean_type(attr: &Vec<Attribute>, bean_type: Option<Type>, bean_type_ident: Option<Ident>) -> Option<BeanType> {
        Self::get_prototype_or_singleton(attr, bean_type, bean_type_ident)
            .map(|bean_type| {
                log_message!("{:?} is the bean type", bean_type);
                bean_type
            })
            .or_else(|| {
                log_message!("Could not find bean type");
                None
            })
    }


}

impl BeanDependencyParser {

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
        field_to_add: AutowiredField,
        lifetime: Option<Lifetime>,
        array_type: Option<TypeArray>,
        injectable_types_builder: &HashMap<String, Bean>,
        fns: &HashMap<TypeId, ModulesFunctions>,
        bean_dep_path: Option<BeanPath>
    ) -> Bean
    {
        log_message!("Adding dependency for {}.", dep_impl.id.clone());

        let autowired_qualifier = field_to_add.clone()
            .qualifier
            .or(Some(field_to_add.type_of_field.to_token_stream().to_string().clone()));

        if autowired_qualifier.is_some() {

            log_message!("Adding dependency {} for bean with id {}.",  SynHelper::get_str(field_to_add.field.clone()), dep_impl.id.clone());

            dep_impl.ident.clone().map(|ident| {
                log_message!("Adding dependency to struct with id {} to struct_impl of name {}", ident.to_string().clone(), dep_impl.id.clone());
            }).or_else(|| {
                log_message!("Could not find ident for {} when attempting to add dependency to struct.", dep_impl.id.clone());
                None
            });

            let bean_type = FnParser::get_fn_for_qualifier(
                     fns,
                    autowired_qualifier.clone(),
                    Some(field_to_add.type_of_field.clone())
                ).map(|fn_type| {
                    BeanParser::get_bean_type_from_qual(autowired_qualifier, None, fn_type)
                })
                .or(None);

            if bean_type.is_some() {
                dep_impl
                    .deps_map
                    .push(DepType {
                        bean_info: field_to_add,
                        lifetime,
                        bean_type: bean_type.flatten(),
                        array_type,
                        bean_type_path: bean_dep_path,
                    });
            } else {
                dep_impl
                    .deps_map
                    .push(DepType {
                        bean_info: field_to_add,
                        lifetime,
                        bean_type: None,
                        array_type,
                        bean_type_path: bean_dep_path,
                    });
            }


        }

        dep_impl
    }
}

pub struct BeanDependencyPathParser;

impl BeanDependencyPathParser {

    fn parse_type_path(path: TypePath) -> BeanPath {
        log_message!("Parsing type segments {}.", path.to_token_stream().to_string().as_str());
        path.qself
            .map(|self_type|
                BeanPath {
                   path_segments: vec![BeanPathParts::QSelfType { q_self: self_type.ty.deref().clone() }]
                }
            )
            .or_else(|| Some(BeanPath {path_segments: Self::parse_path(&path.path)}))
            .unwrap()
    }

    fn parse_path(path: &syn::Path) -> Vec<BeanPathParts> {
        path.segments.iter().flat_map(|segment| {
            match &segment.arguments {
                PathArguments::None => {
                    log_message!("{} type path did not have args.", path.to_token_stream().to_string().as_str());
                    vec![]
                }
                PathArguments::AngleBracketed(angle) => {
                    Self::parse_angle_bracketed(angle, path)
                }
                PathArguments::Parenthesized(parenthesized) => {
                    Self::parse_parenthasized(parenthesized, path)
                }
            }
        }).collect()

    }

    fn parse_parenthasized(parenthesized: &ParenthesizedGenericArguments, path: &syn::Path) -> Vec<BeanPathParts> {
        log_message!("{} are the parenthesized type arguments.", parenthesized.to_token_stream().to_string().as_str());
        let inputs = parenthesized.inputs.iter().map(|arg| {
            arg.clone()
        }).collect::<Vec<Type>>();
        let output = match &parenthesized.output {
            ReturnType::Default => {
                None
            }
            ReturnType::Type(_, o) => {
                Some(o.deref().clone())
            }
        };
        vec![BeanPathParts::FnType {
            input_types: inputs,
            return_type: output,
        }]
    }

    pub fn create_bean_path_parts(in_type: &Type, path: &syn::Path) -> Vec<BeanPathParts> {
        let string = in_type.to_token_stream().to_string();
        let match_ts = string.as_str().clone();
        let path_str = path.to_token_stream().to_string();
        let path_str_to_match = path_str.as_str();
        log_message!("Parsing {} path.", path_str_to_match);
        let mut bean_parts = vec![];
        if path_str_to_match.contains("Arc") && path_str_to_match.contains("Mutex") {
            log_message!("Found arc mutex type {}!", path_str_to_match);
            let type_to_add = BeanPathParts::ArcMutexType {
                arc_mutex_inner_type: in_type.clone(),
                outer_type: path.clone(),
            };
            bean_parts.push(type_to_add);
            Self::add_recurse_parse(in_type, &mut bean_parts);
        } else if path_str_to_match.contains("Arc") {
            log_message!("Found arc type {}!", match_ts);
            let type_to_add = BeanPathParts::ArcType {
                arc_inner_types: in_type.clone(),
                outer_type: path.clone(),
            };
            bean_parts.push(type_to_add);
            Self::add_recurse_parse(in_type, &mut bean_parts);
        } else if path_str_to_match.contains("Mutex") {
            log_message!("Found arc type {}!", match_ts);
            let type_to_add =  BeanPathParts::MutexType {
                mutex_type_inner_type: in_type.clone(),
                outer_type: path.clone(),
            };
            bean_parts.push(type_to_add);
            Self::add_recurse_parse(in_type, &mut bean_parts);
        } else {
            log_message!("Found generic type {}!", match_ts);
            let type_to_add = BeanPathParts::GenType {
                inner: in_type.clone()
            };
            bean_parts.push(type_to_add);
            Self::add_recurse_parse(in_type, &mut bean_parts);
        }
        bean_parts
    }

    fn add_recurse_parse(in_type: &Type, bean_parts: &mut Vec<BeanPathParts>) {
        for p in Self::parse_next_path(in_type).iter() {
            bean_parts.push(p.to_owned());
        }
    }

    fn parse_next_path(in_type: &Type) -> Vec<BeanPathParts> {
        match in_type {
            Type::Path(type_path) => {
                Self::parse_path(&type_path.path.clone())
            }
            _ => {
                vec![]
            }
        }
    }

    fn parse_angle_bracketed(angle: &AngleBracketedGenericArguments, path: &syn::Path) -> Vec<BeanPathParts> {
        log_message!("{} are the angle bracketed type arguments.", angle.to_token_stream().to_string().as_str());
        angle.args.iter().flat_map(|arg| {
            match arg {
                GenericArgument::Type(t) => {
                    log_message!("Found type arg: {}.", SynHelper::get_str(t).as_str());
                    Self::create_bean_path_parts(t, path)
                }
                GenericArgument::Lifetime(_) => {
                    log_message!("Ignored lifetime of generic arg.");
                    vec![]
                }
                GenericArgument::Binding(binding) => {
                    vec![BeanPathParts::BindingType { associated_type: binding.ty.clone() }]
                }
                GenericArgument::Constraint(constraint) => {
                    Self::parse_contraints(constraint, path)
                }
                GenericArgument::Const(_) => {
                    log_message!("Ignored const declaration in generic arg.");
                    vec![]
                }
            }
        }).collect()
    }

    fn parse_contraints(constraint: &Constraint, path: &syn::Path) -> Vec<BeanPathParts> {
        constraint.bounds.iter().flat_map(|bound| {
            match bound {
                TypeParamBound::Trait(trait_bound) => {
                    // let path: syn::Path
                    // trait_bound.path
                    Self::parse_path(&trait_bound.path)
                }
                TypeParamBound::Lifetime(_) => {
                    log_message!("Ignored lifetime contraint when parsing path.");
                    vec![]
                }
            }
        }).collect::<Vec<BeanPathParts>>()
    }
}