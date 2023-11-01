use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::process::id;
use string_utils::strip_whitespace;
use proc_macro2::{Ident, Span};
use quote::__private::ext::RepToTokensExt;
use quote::ToTokens;
use syn::{AngleBracketedGenericArguments, Attribute, Constraint, Field, Fields, GenericArgument, ItemImpl, Lifetime, ParenthesizedGenericArguments, parse2, PathArguments, ReturnType, Type, TypeArray, TypeParamBound, TypePath};
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::{BeanDefinition, BeanPath, BeanPathParts, BeanType};


use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("bean_dependency_path.rs");

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

    pub(crate) fn parse_path_to_bean_path(path: &syn::Path) -> BeanPath {
        BeanPath{
            path_segments: Self::parse_path(path)
        }
    }

    pub(crate) fn is_trait_abstract(item_impl: &Option<ItemImpl>, concrete_ident: &Option<Ident>) -> bool {
        let item_impl_exists = item_impl.as_ref().is_some();
        info!("Testing if trait is abstract.");
        if concrete_ident.as_ref().is_some() {
            info!("Testing if {:?} is abstract.", SynHelper::get_str(concrete_ident.as_ref().unwrap()));
        } else {
            info!("Attempted to test if trat abstract but no concrete ident to compare to.");
            return true
        }
        let is_valid_abstract = item_impl.as_ref().filter(|item_impl_value| {
            let self_ty = item_impl_value.trait_.as_ref();
            if self_ty.as_ref().is_none() {
                return false;
            }
            let trait_ty = &self_ty.cloned().unwrap().1;
            !Self::are_same(trait_ty, concrete_ident.as_ref().unwrap())
        }).is_some();
        if item_impl_exists && is_valid_abstract {
            info!("Trait was abstract.");
            true
        } else {
            info!("Trait was not abstract.");
            false
        }
    }

    pub(crate) fn are_same(trait_ty: &syn::Path, concrete_ident: &syn::Ident) -> bool {
        let bean_path = BeanDependencyPathParser::parse_path_to_bean_path(trait_ty);
        let has_inner = bean_path.get_inner_type().as_ref().is_some();
        if !has_inner {
            info!("Did not have inner.");
            false
        } else {
            let inner_path = bean_path.get_inner_type().unwrap();
            let inner_path = SynHelper::get_str(inner_path.to_token_stream());
            let to_compare_with = SynHelper::get_str(concrete_ident.to_token_stream());
            info!("Testing if {:?} is the same as {:?}, as if it is the same then impl item as only implementing itself.",
                &inner_path, &to_compare_with);
            inner_path == to_compare_with
        }
    }

    pub(crate) fn parse_type(path: Type) -> Option<BeanPath> {
        log_message!("Parsing type segments {}.", path.to_token_stream().to_string().as_str());
        match path {
            Type::Path(tp)  => {
                Some(BeanDependencyPathParser::parse_type_path(tp))
            }
            Type::TraitObject(tt) => {
                let path_segments = tt.bounds.iter().flat_map(|b| match b {
                    TypeParamBound::Trait(trait_bound) => {
                        Self::parse_path(&trait_bound.path)
                    }
                    TypeParamBound::Lifetime(_) => {
                        vec![]
                    }
                }).collect::<Vec<BeanPathParts>>();
                Some(
                    BeanPath {
                        path_segments
                    }
                )

            }
            _ => {
                None
            }
        }
    }

    fn parse_path(path: &syn::Path) -> Vec<BeanPathParts> {
        path.segments.iter().flat_map(|segment| {
            match &segment.arguments {
                PathArguments::None => {
                    log_message!("{} type path did not have args.", path.to_token_stream().to_string().as_str());
                    parse2::<Type>(path.to_token_stream().clone())
                        .ok()
                        .map(|inner| vec![BeanPathParts::GenType { gen_type: inner.clone(), inner_ty_opt: None, outer_ty_opt: Some(inner) }])
                        .or(Some(vec![]))
                        .unwrap()
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

    fn parse_path_inner(path: &syn::Path) -> Vec<BeanPathParts> {
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
            inner_tys: inputs,
            return_type_opt: output,
        }]
    }

    pub fn create_bean_path_parts(in_type: &Type, path: &syn::Path) -> Vec<BeanPathParts> {
        let (match_ts, path_str_to_match) = Self::get_to_match(in_type, path);
        info!("Parsing {} path.", path_str_to_match);
        let mut bean_parts = vec![];
        let pattern = Self::get_patterns(&path_str_to_match);
        info!("Testing pattern: {:?}", pattern);
        let ty_to_add = match pattern.as_slice() {
            ["Arc", "Mutex", ..]  => {
                info!("Found arc mutex type {}!", path_str_to_match);
                BeanPathParts::ArcMutexType {
                    inner_ty: in_type.clone(),
                    outer_path: path.clone(),
                }
            }
            ["Arc", ..] => {
                BeanPathParts::ArcType {
                    inner_ty: in_type.clone(),
                    outer_path: path.clone(),
                }
            }
            ["Mutex", ..] => {
                BeanPathParts::MutexType {
                    inner_ty: in_type.clone(),
                    outer_path: path.clone(),
                }
            }
            ["Box", ..] => {
                BeanPathParts::BoxType {
                    inner_ty: in_type.clone(),
                }
            }
            ["PhantomData", ..] => {
                BeanPathParts::PhantomType {
                    inner_bean_path_parts: BeanPathParts::GenType {
                        gen_type: parse2(path.to_token_stream()).ok().unwrap(),
                        inner_ty_opt: Some(in_type.clone()),
                        outer_ty_opt: parse2("PhantomData".to_token_stream()).ok()
                    }.into()
                }
            }
            _ => {
                info!("Found generic type {}. Setting gen type to {:?}.", match_ts, SynHelper::get_str(path));
                let tokens = path.to_token_stream().to_string();
                let mut outer = None;
                if tokens.contains("<") {
                    let split_tokens = tokens.split("<").collect::<Vec<_>>()[0];
                    info!("Split tokens part {}", split_tokens);
                    let ident_value = Ident::new(
                        strip_whitespace(split_tokens).unwrap(),
                        Span::call_site()
                    );
                    outer = parse2::<Type>(ident_value.to_token_stream())
                        .map(|created| {
                            info!("Parsed first part {:?}", SynHelper::get_str(&created));
                            created
                        })
                        .map_err(|e| {
                            error!("Error parsing tokens {} {:?}", split_tokens, e);
                        })
                        .ok()
                }
                BeanPathParts::GenType {
                    gen_type: parse2::<Type>(path.to_token_stream()).unwrap(),
                    inner_ty_opt: Some(in_type.clone()),
                    outer_ty_opt: outer
                }
            }
        };

        bean_parts.push(ty_to_add);
        Self::add_recurse_parse(in_type, &mut bean_parts);

        bean_parts
    }

    fn get_to_match<'a>(in_type: &Type, path: &syn::Path) -> (String, String) {
        let match_ts = in_type.to_token_stream().to_string();
        let path_str = path.to_token_stream().to_string();
        let path_str_to_match = path_str;
        (match_ts, path_str_to_match)
    }

    fn get_patterns(path_str: &String) -> Vec<&str> {
        let pattern = path_str.split("<").into_iter()
            .flat_map(|s| s.split(">").into_iter())
            .flat_map(|s| strip_whitespace(s).into_iter())
            .collect::<Vec<_>>();
        pattern
    }

    fn add_recurse_parse(in_type: &Type, bean_parts: &mut Vec<BeanPathParts>) {
        for p in Self::parse_next_path(in_type).iter() {
            bean_parts.push(p.to_owned());
        }
    }

    fn parse_next_path(in_type: &Type) -> Vec<BeanPathParts> {
        match in_type {
            Type::Path(type_path) => {
                Self::parse_path_inner(&type_path.path.clone())
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
                    Self::parse_path_inner(&trait_bound.path)
                }
                TypeParamBound::Lifetime(_) => {
                    log_message!("Ignored lifetime contraint when parsing path.");
                    vec![]
                }
            }
        }).collect::<Vec<BeanPathParts>>()
    }
}
