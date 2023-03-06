use syn::{Attribute, ItemFn, ReturnType, Type};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Deref;
use quote::ToTokens;
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::item_parser::ItemParser;
use crate::module_macro_lib::module_tree::{BeanType, FunctionType, ModulesFunctions};
use crate::module_macro_lib::parse_container::ParseContainer;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::module_tree::BeanPathParts::FnType;
use crate::module_macro_lib::util::ParseUtil;


pub struct ItemFnParser;

impl ItemParser<ItemFn> for ItemFnParser {
    fn parse_item(parse_container: &mut ParseContainer, item_fn: &mut ItemFn, path_depth: Vec<String>) {
        let mut path = path_depth.clone();
        path.push(item_fn.sig.ident.to_string().clone());
        Self::item_fn_parse(item_fn.clone())
            .map(|fn_found| {
                parse_container.fns.insert(item_fn.clone().type_id().clone(), ModulesFunctions{ fn_found: fn_found.clone(), path});
            })
            .or_else(|| {
                log_message!("Could not set fn type for fn named: {}", SynHelper::get_str(item_fn.sig.ident.clone()).as_str());
                None
            });
    }
}

impl ItemFnParser {

    pub fn item_fn_parse(fn_found: ItemFn) -> Option<FunctionType> {
        ParseUtil::filter_att(&fn_found.attrs, vec!["#[singleton(", "#[prototype("])
            .iter()
            .flat_map(|attr| {
                match fn_found.sig.output.clone() {
                    ReturnType::Default => {
                        return Self::get_fn_type(&fn_found, attr, None);
                    }
                    ReturnType::Type(_, ty) => {
                        return Self::get_fn_type(&fn_found, attr, Some(ty.deref().clone()))
                    }
                }
            })
            .next()
    }

    fn get_fn_type(fn_found: &ItemFn, attr: &&Attribute, type_ref: Option<Type>) -> Option<FunctionType> {
            SynHelper::get_attr_from_vec_ref(&vec![attr], &vec!["singleton"])
                .map(|singleton_qualifier| FunctionType {
                    item_fn: fn_found.clone(),
                    qualifier: Some(singleton_qualifier),
                    fn_type: type_ref.clone(),
                    bean_type: BeanType::Singleton,
                })
                .or_else(|| {
                    SynHelper::get_attr_from_vec_ref(&vec![attr], &vec!["prototype"])
                        .map(|singleton_qualifier| FunctionType {
                            item_fn: fn_found.clone(),
                            qualifier: Some(singleton_qualifier),
                            fn_type: type_ref,
                            bean_type: BeanType::Singleton,
                        })
                })
    }

    pub(crate) fn get_fn_for_qualifier(
        fns: &HashMap<TypeId, ModulesFunctions>,
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

    fn get_fn_type_by_type(fns: &HashMap<TypeId, ModulesFunctions>, type_of: &Option<Type>) -> Option<FunctionType> {
        let mut next = type_of.as_ref().map(|type_to_check| {
            fns.iter().map(|f| f.1.fn_found.clone())
                .filter(|f| f.fn_type.as_ref()
                    .map(|t| t.to_token_stream().to_string().as_str() == type_to_check.to_token_stream().to_string().as_str())
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
