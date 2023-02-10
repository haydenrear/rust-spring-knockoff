use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use quote::ToTokens;
use syn::{Attribute, ItemFn, ReturnType, Type};
use crate::module_macro_lib::parse_container::ParseContainer;
use crate::module_macro_lib::module_tree::{FunctionType, ModulesFunctions};
use crate::module_macro_lib::util::ParseUtil;

pub struct FnParser;

impl FnParser {

    pub fn to_fn_type(fn_found: ItemFn) -> Option<FunctionType> {
        ParseUtil::filter_singleton_prototype(&fn_found.attrs)
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
        if attr.to_token_stream().to_string().contains("singleton") {
            Some(FunctionType::Singleton(
                fn_found.clone(),
                ParseUtil::strip_value(attr.path.to_token_stream().to_string().as_str())
                    .map(|qual| String::from(qual)),
                type_ref
            ))
        } else if attr.to_token_stream().to_string().contains("prototype") {
            Some(FunctionType::Prototype(
                fn_found.clone(),
                ParseUtil::strip_value(attr.path.to_token_stream().to_string().as_str())
                    .map(|qual| String::from(qual)),
                type_ref
            ))
        } else {
            None
        }
    }

    pub(crate) fn get_fn_for_qualifier(fns: &HashMap<TypeId, ModulesFunctions>, qualifier: Option<String>, type_of: Option<Type>) -> Option<FunctionType> {
        for module_fn_entry in fns {
            match &module_fn_entry.1.fn_found  {
                FunctionType::Singleton(_, fn_qualifier, type_of_fn) => {
                    if type_of.is_some().clone() == type_of_fn.is_some().clone() && type_of.clone().unwrap().to_token_stream().to_string().as_str() == type_of_fn.clone().unwrap().to_token_stream().to_string().as_str() {
                        return Some(module_fn_entry.1.fn_found.clone())
                    } else if qualifier.is_some().clone() && fn_qualifier.is_some().clone() && qualifier.clone().unwrap().as_str() == fn_qualifier.clone().unwrap().as_str() {
                        return Some(module_fn_entry.1.fn_found.clone())
                    }
                }
                FunctionType::Prototype(_, qualifier, _) => {
                    // if fn_qualifier.filter(|qual| qual == qualifier).is_some() {
                    //     return Some(module_fn_entry.1.fn_found.clone())
                    // }
                    //TODO:
                    return None;
                }
            }
        }
        None
    }


}