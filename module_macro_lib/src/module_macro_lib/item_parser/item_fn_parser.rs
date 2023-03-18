use syn::{Attribute, FnArg, ItemFn, Pat, ReturnType, Type};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Deref;
use proc_macro2::Ident;
use quote::ToTokens;
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::item_parser::ItemParser;
use module_macro_shared::parse_container::ParseContainer;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::bean::{BeanPath, BeanType};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use module_macro_shared::bean::BeanPathParts::FnType;
use module_macro_shared::functions::{FunctionType, ModulesFunctions};
use crate::module_macro_lib::bean_parser::bean_dependency_path_parser::BeanDependencyPathParser;
use crate::module_macro_lib::util::ParseUtil;

pub struct ItemFnParser;

impl ItemParser<ItemFn> for ItemFnParser {
    fn parse_item(parse_container: &mut ParseContainer, item_fn: &mut ItemFn, path_depth: Vec<String>) {
        let mut path = path_depth.clone();
        path.push(item_fn.sig.ident.to_string().clone());
        Self::item_fn_parse(item_fn.clone())
            .filter(|fn_found| fn_found.fn_type.as_ref().is_some() && fn_found.fn_type.as_ref().unwrap().get_inner_type().is_some())
            .map(|fn_found| {
                parse_container.fns.insert(fn_found.fn_type.as_ref().unwrap().get_inner_type().as_ref().unwrap().to_token_stream().to_string().clone(),
                                           ModulesFunctions { fn_found: fn_found.clone(), path });
            })
            .or_else(|| {
                log_message!("Could not set fn type for fn named: {}", SynHelper::get_str(item_fn.sig.ident.clone()).as_str());
                None
            });
    }
}

impl ItemFnParser {

    pub fn get_bean_type(fn_found: &ItemFn) -> Option<BeanPath> {
        match &fn_found.sig.output {
            ReturnType::Default => {
                None
            }
            ReturnType::Type(_, ty) => {
                match ty.deref().clone() {
                    Type::Path(type_path) => {
                        Some(BeanDependencyPathParser::parse_type_path(type_path))
                    }
                    _ => {
                        None
                    }
                }
            }
        }
    }

    pub fn item_fn_parse(fn_found: ItemFn) -> Option<FunctionType> {
        Self::get_fn_type(&fn_found, &fn_found.attrs, Self::get_bean_type(&fn_found))
    }

    fn get_fn_type(fn_found: &ItemFn, attr: &Vec<Attribute>, type_ref: Option<BeanPath>) -> Option<FunctionType> {
            SynHelper::get_attr_from_vec(&attr, vec!["singleton"])
                .map(|singleton_qualifier| FunctionType {
                    item_fn: fn_found.clone(),
                    qualifier: Some(singleton_qualifier),
                    fn_type: type_ref.clone(),
                    bean_type: BeanType::Singleton,
                    args: Self::get_injectable_args(fn_found),
                })
                .or_else(|| SynHelper::get_attr_from_vec(&attr, vec!["prototype"])
                        .map(|singleton_qualifier| FunctionType {
                            item_fn: fn_found.clone(),
                            qualifier: Some(singleton_qualifier),
                            fn_type: type_ref,
                            bean_type: BeanType::Prototype,
                            args: Self::get_injectable_args(fn_found),
                        })
                )
    }

    fn get_injectable_args(fn_args: &ItemFn) -> Vec<(Ident, BeanPath)> {
        fn_args.sig.inputs.iter().flat_map(|fn_arg| {
            match fn_arg {
                FnArg::Receiver(_) => {
                    vec![]
                }
                FnArg::Typed(value) => {
                    match &value.ty.deref() {
                        Type::Path(type_path) =>  {
                            Some(BeanDependencyPathParser::parse_type_path(type_path.clone()))
                        }
                        _ => {
                            None
                        }
                    }.map(|type_path| {
                        match value.pat.deref() {
                            Pat::Ident(pat_ident) => {
                                vec!((pat_ident.ident.clone(), type_path))
                            }
                            _ => {
                                vec![]
                            }
                        }
                    })
                        .or(Some(vec![]))
                        .unwrap()
                }
            }
        }).collect::<Vec<(Ident, BeanPath)>>()
    }

    pub(crate) fn get_fn_for_qualifier(
        fns: &HashMap<String, ModulesFunctions>,
        qualifier: &Option<String>,
        type_of: &Option<Type>
    ) -> Option<FunctionType> {
        qualifier.as_ref().map(|qualifier_to_match|
            fns.iter()
                .filter(|fn_to_check| fn_to_check.1.fn_found.qualifier
                    .as_ref()
                    .map(|qual| qualifier_to_match == qual)
                    .or(Some(false)).unwrap()
                )
                .next()
                .map(|f| f.1.fn_found.clone())
        )
            .flatten()
            .or_else(|| Self::get_fn_type_by_type(fns, type_of))
    }

    fn get_fn_type_by_type(fns: &HashMap<String, ModulesFunctions>, type_of: &Option<Type>) -> Option<FunctionType> {
        let mut next = type_of.as_ref().map(|type_to_check| {
            fns.iter().map(|f| f.1.fn_found.clone())
                .filter(|f| f.fn_type.as_ref()
                    .map(|t| t.get_inner_type().to_token_stream().to_string().as_str() == type_to_check.to_token_stream().to_string().as_str())
                    .or(Some(false)).unwrap()
                )
                .map(|fn_type| fn_type.clone())
                .collect::<Vec<FunctionType>>()
        })
            .or(Some(vec![]))
            .unwrap();
        next.pop()

    }
}
