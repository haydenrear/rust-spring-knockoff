use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::str::pattern::Pattern;
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf, TypeArray, ItemEnum, ReturnType};
use syn::__private::str;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    LitStr,
    Token,
    Ident,
    token::Paren,
};
use quote::{quote, format_ident, IdentFragment, ToTokens, quote_token, TokenStreamExt};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use crate::module_macro_lib::module_parser::parse_item;
use crate::module_macro_lib::module_tree::{DepImpl, Trait, Profile, DepType, BeanType, BeanDefinition, AutowiredField};
use crate::module_macro_lib::spring_knockoff_context::ApplicationContextGenerator;


/**
Will be annotated with #[bean] and #[singleton], #[prototype] as provided factory functions.
 **/
pub struct ModulesFunctions {
    pub fn_found: FunctionType
}

#[derive(Clone)]
pub enum FunctionType {
    Singleton(ItemFn, Option<String>, Option<Type>),
    Prototype(ItemFn, Option<String>, Option<Type>)
}

pub struct ParseContainer {
    pub injectable_types: HashMap<String, DepImpl>,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<TypeId, ModulesFunctions>,
    pub profiles: Vec<Profile>
}

impl ParseContainer {
    /**
    Generate the token stream from the created ModuleContainer tree.
     **/
    pub fn to_token_stream(&self) -> proc_macro2::TokenStream {

        self.log_app_container_info();

        let mut token = quote! {};

        for token_type in &self.injectable_types {
            println!("Implementing container for {} if is not none and implements Default.", token_type.1.id.clone());
            if token_type.1.struct_type.is_some() || token_type.1.ident.is_some() {


                let field_types = token_type.1.deps_map
                    .clone().iter()
                    .map(|d| d.bean_info.type_of_field.clone())
                    .collect::<Vec<Type>>();

                let identifiers = token_type.1.deps_map
                    .clone().iter()
                    .flat_map(|d| {
                        match &d.bean_info.field.ident {
                            None => {
                                vec![]
                            }
                            Some(identifier) => {
                                vec![identifier.clone()]
                            }
                        }
                    })
                    .collect::<Vec<Ident>>();



                // if token_type.1.struct_type.is_some() {
                //
                //     let struct_type =  token_type.1.struct_type.clone()
                //         .unwrap();
                //
                //     println!("Implementing container for {}.", struct_type.to_token_stream().to_string().clone());
                //
                //     let this_struct_impl = ApplicationContextGenerator::gen_autowire_code(
                //         field_types, identifiers, struct_type
                //     );
                //     token.append_all(this_struct_impl);
                // } else {
                if token_type.1.ident.is_some() {
                    let struct_type =  token_type.1.ident.clone()
                        .unwrap();

                    println!("Implementing container for {}.", struct_type.to_token_stream().to_string().clone());

                    let this_struct_impl = ApplicationContextGenerator::gen_autowire_code_ident(
                        field_types, identifiers, struct_type
                    );
                    token.append_all(this_struct_impl);

                }


            }
        }

        let deps = self.injectable_types.values()
            .into_iter()
            .collect::<Vec<&DepImpl>>();

        let listable_bean_factory = ApplicationContextGenerator::new_listable_bean_factory(deps);

        token.extend(listable_bean_factory.into_iter());

        token
    }

    /**
    1. Make sure that there are no cyclic dependencies.
    2. Reorder so that the beans are added to the container in the correct order.
    **/
    pub fn is_valid_ordering_create(&self) -> Vec<String> {
        let mut already_processed = vec![];
        for i_type in self.injectable_types.iter() {
            if !self.is_valid_ordering(&mut already_processed, i_type.1) {
                println!("Was not valid ordering!");
            }
        }
        already_processed
    }

    pub fn is_valid_ordering(&self, already_processed: &mut Vec<String>, dep: &DepImpl) -> bool {
        for dep_impl in &dep.deps_map {
            if already_processed.contains(&ParseContainer::get_identifier(dep_impl)) {
                continue;
            }
            if !self.injectable_types.get(&dep.id)
                .map(|next| {
                    if self.is_valid_ordering(already_processed, next) {
                        already_processed.push(next.id.clone());
                        return true;
                    }
                    false
                })
                .or(Some(false))
                .unwrap() {
                return false;
            }
        }
        true
    }

    pub fn get_identifier(dep_type: &DepType) -> String {
        match &dep_type.bean_info.qualifier  {
            None => {
                dep_type.bean_info.type_of_field.to_token_stream().to_string().clone()
            }
            Some(qual) => {
                qual.clone()
            }
        }
    }

    pub fn log_app_container_info(&self) {
        self.injectable_types.iter().filter(|&s| s.1.struct_found.is_none())
            .for_each(|s| {
                println!("Could not find struct type with ident {}.", s.0.clone());
            })
    }

    /**
    Add the struct and the impl from the ItemImpl
     **/
    pub fn create_update_impl(&mut self, item_impl: &mut ItemImpl) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        &mut self.injectable_types.get_mut(&id)
            .map(|struct_impl: &mut DepImpl| {
                struct_impl.traits_impl.push(item_impl.clone());
            })
            .or_else(|| {
                let impl_found = DepImpl {
                    struct_type: Some(item_impl.self_ty.deref().clone()),
                    struct_found: None,
                    traits_impl: vec![item_impl.clone()],
                    enum_found: None,
                    attr: vec![],
                    deps_map: vec![],
                    id: id.clone(),
                    profile: vec![],
                    ident: None,
                    fields: vec![],
                };
                self.injectable_types.insert(id, impl_found);
                None
            });
    }

    pub fn add_item_struct(&mut self, item_impl: &mut ItemStruct) {
        println!("adding type with name {}", item_impl.ident.clone().to_token_stream().to_string());
        println!("adding type with name {}", item_impl.to_token_stream().to_string().clone());
        self.injectable_types.get_mut(&item_impl.ident.to_string().clone())
            .map(|struct_impl: &mut DepImpl| {
                struct_impl.struct_found = Some(item_impl.clone());
            })
            .or_else(|| {
                let mut impl_found = DepImpl {
                    struct_type: None,
                    struct_found: Some(item_impl.clone()),
                    traits_impl: vec![],
                    enum_found: None,
                    attr: vec![],
                    deps_map: vec![],
                    id: item_impl.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(item_impl.ident.clone()),
                    fields: vec![item_impl.fields.clone()],
                };
                self.set_deps(&mut impl_found);
                self.injectable_types.insert(item_impl.ident.to_string().clone(), impl_found);
                None
            });
    }

    pub fn add_item_enum(&mut self, enum_to_add: &mut ItemEnum) {
        println!("adding type with name {}", enum_to_add.ident.clone().to_token_stream().to_string());
        &mut self.injectable_types.get_mut(&enum_to_add.ident.to_string().clone())
            .map(|struct_impl: &mut DepImpl| {
                struct_impl.enum_found = Some(enum_to_add.clone());
            })
            .or_else(|| {
                let enum_fields = enum_to_add.variants.iter()
                    .map(|variant| variant.fields.clone())
                    .collect::<Vec<Fields>>();
                let mut impl_found = DepImpl {
                    struct_type: None,
                    struct_found: None,
                    traits_impl: vec![],
                    enum_found: Some(enum_to_add.clone()),
                    attr: vec![],
                    deps_map: vec![],
                    id: enum_to_add.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(enum_to_add.ident.clone()),
                    fields: enum_fields,
                };
                self.set_deps(&mut impl_found);
                self.injectable_types.insert(enum_to_add.ident.to_string().clone(), impl_found);
                None
            });
    }

    pub fn create_update_trait(&mut self, trait_found: &mut ItemTrait) {
        if !self.traits.contains_key(&trait_found.ident.to_string().clone()) {
            self.traits.insert(trait_found.ident.to_string().clone(), Trait::new(trait_found.clone()));
        } else {
            println!("Contained trait already!");
        }
    }

    pub fn set_deps(&mut self, dep_impl: &mut DepImpl) {
        dep_impl.fields.clone().iter().for_each(|fields| {
            match fields.clone() {
                Fields::Named(fields_named) => {
                    fields_named.named.iter().for_each(|field: &Field| {
                        field.clone().ident.map(|ident: Ident| {
                            println!("found field {}.", ident.to_string().clone());
                        });
                        println!("{} is the field type!", field.ty.to_token_stream().clone());
                        self.match_ty_add_dep(
                            dep_impl,
                            None,
                            None,
                            field.clone()
                        );
                    });
                }
                Fields::Unnamed(unnamed_field) => {}
                _ => {}
            };
        });
    }

    /**
    Adds the field to the to the tree as a dependency. Replace with DepImpl...
    **/
    pub fn match_ty_add_dep(
        &mut self,
        dep_impl: &mut DepImpl,
        lifetime: Option<Lifetime>,
        array_type: Option<TypeArray>,
        field: Field,
    ) {
        ParseContainer::get_autowired_field_dep(field.attrs.clone(), field.clone())
            .map(|autowired| {
                println!("Found field with type {}.", autowired.field.ty.to_token_stream().to_string().clone());
                if autowired.field.ident.is_some() {
                    println!("Found field with ident {}.", autowired.field.ident.to_token_stream().to_string().clone());
                }
                match field.ty.clone() {
                    Type::Array(arr) => {
                        println!("found array type {}.", arr.to_token_stream().to_string().clone());
                        self.add_type_dep(dep_impl, autowired, lifetime, Some(arr));
                    }
                    Type::Group(grp) => {
                        println!("found group type {}.", grp.to_token_stream().to_string().clone());
                    }
                    Type::Infer(grp) => {
                        println!("found infer type {}.", grp.to_token_stream().to_string().clone());
                    }
                    Type::Macro(grp) => {
                        println!("found macro type {}.", grp.to_token_stream().to_string().clone());
                    }
                    Type::Paren(grp) => {
                        println!("found paren type {}.", grp.to_token_stream().to_string().clone());
                    }
                    Type::Path(path) => {
                        println!("Not adding path: {} yet.", path.path.to_token_stream().to_string().clone());
                        path.qself.map(|q_self| {
                            println!("Asserting that {} and {} are the same.", q_self.ty.clone().to_token_stream().clone(), field.clone().to_token_stream().to_string().clone());
                        });
                        // self.add_type_dep();
                    }
                    Type::Reference(reference_found) => {
                        let ref_type = reference_found.elem.clone();
                        println!("{} is the ref type", ref_type.to_token_stream());
                        if lifetime.is_some() {
                            println!("Cannot add nested references - failed to add autowired field {} to container.", autowired.field.to_token_stream().to_string().clone());
                        } else {
                            self.add_type_dep(dep_impl, autowired, reference_found.lifetime, array_type);
                        }
                    }
                    Type::Slice(grp) => {
                        println!("found slice type {}.", grp.to_token_stream().to_string().clone());
                    }
                    Type::TraitObject(grp) => {
                        println!("found trait object type {}.", grp.to_token_stream().to_string().clone());
                    }
                    Type::Tuple(grp) => {
                        println!("found tuple type {}.", grp.to_token_stream().to_string().clone());
                    }
                    some_other => {
                        println!("found type {} but did not add.", some_other.to_token_stream().to_string().clone());
                    }
                };
            });
    }

    pub fn add_fn_to_dep_types(&mut self, item_fn: &mut ItemFn) {
        ParseContainer::get_fn_type(item_fn.clone())
            .map(|fn_found| {
                self.fns.insert(item_fn.clone().type_id().clone(), ModulesFunctions{ fn_found: fn_found.clone() });
                for i_type in self.injectable_types.iter_mut() {
                    for dep_type in i_type.1.deps_map.iter_mut() {
                        if dep_type.bean_type.is_none() {
                            match &fn_found {
                                FunctionType::Singleton(fn_type, qualifier, type_found) => {
                                    dep_type.bean_type = Some(
                                        BeanType::Singleton(
                                            BeanDefinition {
                                            qualifier: qualifier.clone(),
                                                bean_type_type: type_found.clone(),
                                            },
                                            Some(fn_found.clone()))
                                    );
                                }
                                FunctionType::Prototype(fn_type, qualifier, type_found) => {
                                    dep_type.bean_type = Some(
                                        BeanType::Prototype(
                                        BeanDefinition {
                                            qualifier: qualifier.clone(),
                                            bean_type_type: type_found.clone()
                                        },
                                        Some(fn_found.clone())
                                    ));
                                }
                            };
                        }
                    }
                }
            });
    }


    pub fn add_type_dep(
        &mut self, dep_impl: &mut DepImpl, field_to_add: AutowiredField, lifetime: Option<Lifetime>, array_type: Option<TypeArray>
    )
    {
        let type_dep = &field_to_add.field.clone().ty.to_token_stream().to_string();
        let contains_key = self.injectable_types.contains_key(type_dep);
        let struct_exists = self.injectable_types.get_mut(&field_to_add.field.clone().ty.to_token_stream().to_string()).is_some();
        let autowired_qualifier = field_to_add.qualifier.clone().is_some();
        if autowired_qualifier && contains_key && struct_exists {

            dep_impl.ident.clone().map(|ident| {
                println!("Adding dependency with id {} to struct_impl of name {}", dep_impl.id.clone(), ident.to_string().clone());
            }).or_else(|| {
                println!("Could not find ident for {}.", dep_impl.id.clone());
                None
            });

            let bean_type = self.get_fn_for_qualifier(
                field_to_add.qualifier.clone(),
                Some(field_to_add.type_of_field.clone())
            ).map(|fn_type| {
                ParseContainer::get_bean_type_from_qual(field_to_add.qualifier.clone(), None, fn_type)
            })
                .or(None);

            if bean_type.is_some() {
                self.injectable_types.get_mut(dep_impl.id.as_str())
                    .unwrap()
                    .deps_map
                    .push(DepType {
                        bean_info: field_to_add,
                        lifetime,
                        bean_type: bean_type.unwrap(),
                        array_type
                    });
            } else {
                self.injectable_types.get_mut(dep_impl.id.as_str())
                    .unwrap()
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
    }

    fn get_fn_for_qualifier(&self, qualifier: Option<String>, type_of: Option<Type>) -> Option<FunctionType> {
        for module_fn_entry in &self.fns {
            match &module_fn_entry.1.fn_found  {
                FunctionType::Singleton(_, fn_qualifier, type_of_fn) => {
                    if type_of.clone().is_some() && type_of_fn.clone().is_some() && type_of.clone().unwrap().to_token_stream().to_string().as_str() == type_of_fn.clone().unwrap().to_token_stream().to_string().as_str() {
                        return Some(module_fn_entry.1.fn_found.clone())
                    } else if qualifier.clone().is_some() && fn_qualifier.clone().is_some() && qualifier.clone().unwrap() == fn_qualifier.clone().unwrap() {
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

    fn get_fn_type(fn_found: ItemFn) -> Option<FunctionType> {
        fn_found.attrs.iter()
            .filter(|attr| {
                let attr_name = attr.to_token_stream().to_string();
                println!("Checking attribute: {} for fn.", attr_name.clone());
                attr_name.contains("singleton") || attr_name.contains("prototype")
            })
            .flat_map(|attr| {
                    match fn_found.sig.output.clone() {
                        ReturnType::Default => {
                            if attr.to_token_stream().to_string().contains("singleton") {
                                Some(FunctionType::Singleton(
                                    fn_found.clone(),
                                    ParseContainer::strip_value(attr.path.to_token_stream().to_string())
                                        .map(|qual| String::from(qual)),
                                    None
                                ))
                            } else if attr.to_token_stream().to_string().contains("prototype") {
                                Some(FunctionType::Prototype(
                                    fn_found.clone(),
                                    ParseContainer::strip_value(attr.path.to_token_stream().to_string())
                                        .map(|qual| String::from(qual)),
                                    None
                                ))
                            } else {
                                None
                            }
                        }
                        ReturnType::Type(_, ty) => {
                            if attr.to_token_stream().to_string().contains("singleton") {
                                Some(FunctionType::Singleton(
                                    fn_found.clone(),
                                    ParseContainer::strip_value(attr.path.to_token_stream().to_string())
                                        .map(|qual| String::from(qual)),
                                    Some(ty.deref().clone())
                                ))
                            } else if attr.to_token_stream().to_string().contains("prototype") {
                                Some(FunctionType::Prototype(
                                    fn_found.clone(),
                                    ParseContainer::strip_value(attr.path.to_token_stream().to_string())
                                        .map(|qual| String::from(qual)),
                                    Some(ty.deref().clone())
                                ))
                            } else {
                                None
                            }
                        }
                    }
            })
            .next()
    }

    pub fn get_autowired_field_dep(attrs: Vec<Attribute>, field: Field) -> Option<AutowiredField> {
        println!("Checking attributes for field {}.", field.to_token_stream().to_string().clone());
        attrs.iter().map(|attr| {
            println!("Checking attribute: {} for field.", attr.to_token_stream().to_string().clone());
            ParseContainer::get_qualifier_from_autowired(attr.clone())
                .map(|autowired_value| {
                    AutowiredField{
                        qualifier: None,
                        lazy: false,
                        field: field.clone(),
                        type_of_field: field.ty.clone(),
                    }
                })
        }).next().unwrap_or_else(|| {
            println!("Could not create autowired field of type {}.", field.ty.to_token_stream().to_string().clone());
            None
        })
    }

    pub fn strip_value(value: String) -> Option<String> {
        value.strip_prefix("(")
            .map(|without_prefix| without_prefix.strip_suffix(")")
                .map(|str| String::from(str))
                .or(None)
            ).unwrap_or(None)
    }

    pub fn get_qualifier_from_autowired(autowired_attr: Attribute) -> Option<String> {
        if autowired_attr.path.to_token_stream().to_string().clone().contains("autowired") {
            return ParseContainer::strip_value(autowired_attr.path.to_token_stream().to_string().clone());
        }
        None
    }

    pub fn get_bean_type_from_factory_fn(attrs: Vec<Attribute>, module_fn: ModulesFunctions) -> Option<BeanType> {
        if attrs.iter().any(|attr| {
            let attr_str = attr.to_token_stream().to_string();
            attr_str.contains("bean") || attr_str.contains("singleton") || attr_str.contains("prototype")
        }) {
            return attrs.iter().flat_map(|attr| {
                let qualifier = ParseContainer::strip_value(attr.path.to_token_stream().to_string().clone());
                if attr.to_token_stream().to_string().contains("singleton") {
                    return Some(
                        BeanType::Singleton(
                            BeanDefinition{
                                qualifier,
                                bean_type_type: ParseContainer::get_type_from_fn_type(&module_fn.fn_found)
                            },
                            Some(module_fn.fn_found.clone())
                        ));
                } else if attr.to_token_stream().to_string().contains("prototype") {
                    return Some(BeanType::Prototype(
                        BeanDefinition{
                            qualifier,
                            bean_type_type: ParseContainer::get_type_from_fn_type(&module_fn.fn_found)
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
                            bean_type_type: ParseContainer::get_type_from_fn_type(&module_fn)
                        },
                        Some(module_fn)
                    ));
            }
            FunctionType::Prototype(_, qualifier_found, _) => {
                return Some(BeanType::Prototype(
                    BeanDefinition{
                        qualifier: qualifier_found.clone(),
                        bean_type_type: ParseContainer::get_type_from_fn_type(&module_fn)
                    },
                    Some(module_fn)
                ));
            }
        }
    }

    pub fn get_type_from_fn_type(fn_type: &FunctionType) -> Option<Type> {
        match fn_type {
            FunctionType::Singleton(_, _, ty) => {
                ty.clone()
            }
            FunctionType::Prototype(_, _, ty) => {
                ty.clone()
            }
        }
    }



}
