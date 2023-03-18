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
use module_macro_shared::bean::{Bean, BeanPath, BeanPathParts, BeanType};
use module_macro_shared::dependency::{AutowiredField, DepType};
use module_macro_shared::functions::{FunctionType, ModulesFunctions};
use crate::module_macro_lib::item_parser::item_fn_parser::ItemFnParser;
use module_macro_shared::parse_container::ParseContainer;
use crate::module_macro_lib::module_tree::BeanDefinition;
use crate::module_macro_lib::util::ParseUtil;


use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;


pub struct BeanDependencyPathParser;

impl BeanDependencyPathParser {

    pub(crate) fn parse_type_path(path: TypePath) -> BeanPath {
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
                    Self::parse_parenthesized(parenthesized, path)
                }
            }
        }).collect()

    }

    fn parse_parenthesized(parenthesized: &ParenthesizedGenericArguments, path: &syn::Path) -> Vec<BeanPathParts> {
        log_message!("{} are the parenthesized type arguments.", parenthesized.to_token_stream().to_string().as_str());
        let inputs = parenthesized.inputs.iter().map(|arg| {
            arg.clone()
        }).collect::<Vec<Type>>();
        let output = match &parenthesized.output {
            ReturnType::Default => {
                None
            }
            ReturnType::Type(_, box_type) => {
                Some(box_type.deref().clone())
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
