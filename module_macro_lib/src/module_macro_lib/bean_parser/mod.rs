use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::Ident;
use quote::__private::ext::RepToTokensExt;
use quote::ToTokens;
use syn::{AngleBracketedGenericArguments, Attribute, Constraint, Field, Fields, GenericArgument, Lifetime, ParenthesizedGenericArguments, PathArguments, ReturnType, Type, TypeArray, TypeParamBound, TypePath};
use crate::module_macro_lib::fn_parser::FnParser;
use crate::module_macro_lib::parse_container::ParseContainer;
use crate::module_macro_lib::module_tree::{AutowiredField, Bean, BeanDefinition, BeanPath, BeanPathParts, BeanType, DepType, FunctionType, ModulesFunctions};
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
                    for mut field in fields_named.named.iter() {
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
                        dep_impl = Self::add_type_dep(dep_impl, autowired, lifetime, Some(arr), injectable_types_builder, fns, None);
                    }
                    Type::Path(path) => {
                        let type_path = BeanDependencyPathParser::parse_type_path(path);
                        dep_impl = Self::add_type_dep(dep_impl, autowired, lifetime, array_type, injectable_types_builder, fns, Some(type_path));
                    }
                    Type::Reference(reference_found) => {
                        let ref_type = reference_found.elem.clone();
                        println!("{} is the ref type", ref_type.to_token_stream());
                        dep_impl = Self::add_type_dep(dep_impl, autowired, reference_found.lifetime, array_type, injectable_types_builder, fns, None);
                    }
                    other => {
                        println!("{} is the other type", other.to_token_stream().to_string().as_str());
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

pub struct BeanDependencyPathParser;

impl BeanDependencyPathParser {

    fn parse_type_path(path: TypePath) -> BeanPath {
        println!("Parsing type segments {}.", path.to_token_stream().to_string().as_str());
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
                    println!("{} type path did not have args.", path.to_token_stream().to_string().as_str());
                    vec![]
                }
                PathArguments::AngleBracketed(angle) => {
                    Self::parse_angle_bracketed(angle)
                }
                PathArguments::Parenthesized(parenthasized) => {
                    Self::parse_parenthasized(parenthasized)
                }
            }
        }).collect()

    }

    fn parse_parenthasized(parenthesized: &ParenthesizedGenericArguments) -> Vec<BeanPathParts> {
        println!("{} are the parenthesized type arguments.", parenthesized.to_token_stream().to_string().as_str());
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

    fn parse_angle_bracketed(angle: &AngleBracketedGenericArguments) -> Vec<BeanPathParts> {
        println!("{} are the angle bracketed type arguments.", angle.to_token_stream().to_string().as_str());
        angle.args.iter().flat_map(|arg| {
            match arg {
                GenericArgument::Type(t) => {
                    println!("Found type of generic arg: {}", t.to_token_stream().to_string().as_str());
                    if t.to_token_stream().to_string().as_str().contains("Arc") {
                        return vec![BeanPathParts::ArcType { arc_inner_types: t.clone() }]
                    }
                    vec![BeanPathParts::GenType {inner: t.clone()}]
                }
                GenericArgument::Lifetime(_) => {
                    println!("Ignored lifetime of generic arg.");
                    vec![]
                }
                GenericArgument::Binding(binding) => {
                    vec![BeanPathParts::BindingType { associated_type: binding.ty.clone() }]
                }
                GenericArgument::Constraint(constraint) => {
                    Self::parse_contraints(constraint)
                }
                GenericArgument::Const(_) => {
                    println!("Ignored const declaration in generic arg.");
                    vec![]
                }
            }
        }).collect()
    }

    fn parse_contraints(constraint: &Constraint) -> Vec<BeanPathParts> {
        constraint.bounds.iter().flat_map(|bound| {
            match bound {
                TypeParamBound::Trait(trait_bound) => {
                    // let path: syn::Path
                    // trait_bound.path
                    Self::parse_path(&trait_bound.path)
                }
                TypeParamBound::Lifetime(_) => {
                    println!("Ignored lifetime contraint when parsing path.");
                    vec![]
                }
            }
        }).collect::<Vec<BeanPathParts>>()
    }
}