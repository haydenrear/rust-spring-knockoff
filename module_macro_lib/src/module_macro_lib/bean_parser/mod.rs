use std::any::TypeId;
use std::collections::HashMap;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{Attribute, Field, Fields, Lifetime, Type, TypeArray};
use crate::module_macro_lib::fn_parser::FnParser;
use crate::module_macro_lib::parse_container::ParseContainer;
use crate::module_macro_lib::module_tree::{AutowiredField, Bean, BeanDefinition, BeanType, DepType, FunctionType, ModulesFunctions};
use crate::module_macro_lib::util::ParseUtil;

pub struct BeanParser;
pub struct BeanDependencyParser;

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

impl BeanDependencyParser {

    pub fn add_dependencies(
        mut bean: Bean,
        injectable_types_builder: &HashMap<String, Bean>,
        fns: &HashMap<TypeId, ModulesFunctions>
    ) -> Bean {
        for fields in bean.fields.clone().iter() {
            match fields.clone() {
                Fields::Named(fields_named) => {
                    for field in fields_named.named.iter() {
                        field.clone().ident.map(|ident: Ident| {
                            println!("found field {}.", ident.to_string().clone());
                        });
                        println!("{} is the field type!", field.ty.to_token_stream().clone());
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
        let autowired = ParseContainer::get_autowired_field_dep(field.attrs.clone(), field.clone());
        match autowired {
            None => {
                dep_impl
            }
            Some(autowired) => {
                println!("Found field with type {}.", autowired.field.ty.to_token_stream().to_string().clone());
                if autowired.field.ident.is_some() {
                    println!("Found field with ident {}.", autowired.field.ident.to_token_stream().to_string().clone());
                }
                match field.ty.clone() {
                    Type::Array(arr) => {
                        println!("found array type {}.", arr.to_token_stream().to_string().clone());
                        dep_impl = Self::add_type_dep(dep_impl, autowired, lifetime, Some(arr), injectable_types_builder, fns);
                    }
                    Type::Path(path) => {
                        println!("Adding type path.");
                        //TODO: extension point for lazy
                        dep_impl = Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns);
                    }
                    Type::Reference(reference_found) => {
                        let ref_type = reference_found.elem.clone();
                        println!("{} is the ref type", ref_type.to_token_stream());
                        dep_impl = Self::add_type_dep(dep_impl, autowired, reference_found.lifetime, array_type, injectable_types_builder, fns);
                    }
                    _ => {
                        dep_impl = Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns)
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
        fns: &HashMap<TypeId, ModulesFunctions>
    ) -> Bean
    {
        println!("Adding dependency for {}.", dep_impl.id.clone());
        let type_dep = &field_to_add.field.clone().ty.to_token_stream().to_string();
        let contains_key = injectable_types_builder.contains_key(type_dep);
        let struct_exists = injectable_types_builder.get(&field_to_add.field.clone().ty.to_token_stream().to_string()).is_some();
        let autowired_qualifier = field_to_add.clone().qualifier.or(Some(field_to_add.type_of_field.to_token_stream().to_string().clone()));
        if autowired_qualifier.is_some() && contains_key && struct_exists {

            dep_impl.ident.clone().map(|ident| {
                println!("Adding dependency with id {} to struct_impl of name {}", dep_impl.id.clone(), ident.to_string().clone());
            }).or_else(|| {
                println!("Could not find ident for {}.", dep_impl.id.clone());
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
                        bean_type: bean_type.unwrap(),
                        array_type
                    });
            } else {
                dep_impl
                    .deps_map
                    .push(DepType {
                        bean_info: field_to_add,
                        lifetime,
                        bean_type: None,
                        array_type
                    });
            }


        } else {
            if !struct_exists {
                println!("Struct impl did not exist in module container.")
            }
            if !contains_key {
                println!("Dependency did not exist in module container.")
            }
        }

        dep_impl
    }


}