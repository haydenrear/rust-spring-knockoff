use proc_macro2::Ident;
use quote::ToTokens;
use syn::{Attribute, Type};
use crate::module_macro_lib::app_container::ParseContainer;
use crate::module_macro_lib::module_tree::{BeanDefinition, BeanType, FunctionType, ModulesFunctions};
use crate::module_macro_lib::util::ParseUtil;

pub struct BeanParser;

impl BeanParser {
    pub(crate) fn get_prototype_or_singleton(attr: &Vec<Attribute>, bean_type: Option<Type>, bean_type_ident: Option<Ident>) -> Option<BeanType> {
        ParseUtil::filter_singleton_prototype(attr)
            .and_then(|s| {
                let qualifier = ParseUtil::strip_value_attr(s);

                qualifier.iter()
                    .for_each(|qual|
                        println!("Found bean with qualifier {}.", qual)
                    );

                println!("Found bean with attr {}.", s.to_token_stream().to_string().as_str());
                if s.path.to_token_stream().to_string().as_str().contains("singleton") {
                    return Some(BeanType::Singleton(BeanDefinition{
                        qualifier: qualifier,
                        bean_type_type: bean_type,
                        bean_type_ident
                    }, None))
                        .map(|bean_type| {
                            println!("Found singleton bean: {:?}.", bean_type);
                            bean_type
                        })
                } else if s.path.to_token_stream().to_string().as_str().contains("prototype") {
                    return Some(BeanType::Prototype(BeanDefinition{
                        qualifier: qualifier,
                        bean_type_type: bean_type,
                        bean_type_ident
                    }, None))
                        .map(|bean_type| {
                            println!("Found singleton bean: {:?}.", bean_type);
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
                let qualifier = ParseUtil::strip_value(attr.path.to_token_stream().to_string().as_str());
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
                println!("{:?} is the bean type", bean_type);
                bean_type
            })
            .or_else(|| {
                println!("Could not find bean type");
                None
            })
    }


}