use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::str::pattern::Pattern;
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf, TypeArray, ItemEnum};
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

pub enum FunctionType {
    Singleton(ItemFn),
    Prototype(ItemFn)
}

pub struct ParseContainer {
    pub injectable_types: HashMap<String, DepImpl>,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<String, ModulesFunctions>,
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

                if token_type.1.struct_type.is_some() {

                    let struct_type =  token_type.1.struct_type.clone()
                        .unwrap();

                    println!("Implementing container for {}.", struct_type.to_token_stream().to_string().clone());

                    let this_struct_impl = ApplicationContextGenerator::gen_autowire_code(
                        field_types, identifiers, struct_type
                    );
                    token.append_all(this_struct_impl);
                } else {
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

        token
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

    pub fn add_fn(&mut self, item_fn: &mut ItemFn) {
        ParseContainer::get_fn_type(item_fn.clone())
            .map(|fn_found| {
                self.fns.insert(item_fn.to_token_stream().to_string().clone(), ModulesFunctions{fn_found});
            });
    }


    pub fn add_type_dep(
        &mut self, dep_impl: &mut DepImpl, field_to_add: AutowiredField, lifetime: Option<Lifetime>, array_type: Option<TypeArray>
    )
    {
        let type_dep = &field_to_add.field.clone().ty.to_token_stream().to_string();
        //TODO: is the ident equal to ty?
        let contains_key = self.injectable_types.contains_key(type_dep);
        let struct_exists = self.injectable_types.get_mut(&field_to_add.field.clone().ty.to_token_stream().to_string()).is_some();
        let autowired_qualifier = field_to_add.qualifier.clone().is_some();
        if autowired_qualifier && contains_key && struct_exists {
            let id = self.injectable_types.get(field_to_add.qualifier.clone().unwrap().as_str())
                .and_then(|struct_found: &DepImpl| Some((struct_found.id.clone())))
                .or(Some(field_to_add.qualifier.clone().unwrap()))
                .unwrap();
            dep_impl.ident.clone().map(|ident| {
                println!("Adding dependency with id {} to struct_impl of name {}", dep_impl.id.clone(), ident.to_string().clone());
            }).or_else(|| {
                println!("Could not find ident for {}.", dep_impl.id.clone());
                None
            });
            self.injectable_types.get_mut(dep_impl.id.as_str())
                .unwrap()
                .deps_map
                .push(DepType {
                    bean_info: field_to_add,
                    lifetime,
                    bean_type: None,
                    array_type
                });
        } else {
            if !struct_exists {
                println!("Struct impl did not exist in module container.")
            }
            if !contains_key {
                println!("Dependency did not exist in module container.")
            }
        }
    }

    fn get_fn_type(fn_found: ItemFn) -> Option<FunctionType> {
        fn_found.attrs.iter()
            .filter(|attr| {
                println!("Checking attribuge: {} for fn.", attr.to_token_stream().to_string().clone());
                attr.to_token_stream().to_string().contains("bean")
            })
            .flat_map(|attr| {
                if attr.to_token_stream().to_string().contains("singleton") {
                    Some(FunctionType::Singleton(fn_found.clone()))
                } else if attr.to_token_stream().to_string().contains("prototype") {
                    Some(FunctionType::Prototype(fn_found.clone()))
                } else {
                   None
                }
            })
            .next()
    }

    pub fn get_autowired_field_dep(attrs: Vec<Attribute>, field: Field) -> Option<AutowiredField> {
        println!("Checking attribuges for field {}.", field.to_token_stream().to_string().clone());
        attrs.iter().map(|attr| {
            println!("Checking attribuge: {} for field.", attr.to_token_stream().to_string().clone());
            ParseContainer::get_qualifier_from_autowired(attr.clone())
                .map(|autowired_value| {
                    AutowiredField{
                        qualifier: None,
                        lazy: false,
                        field: field.clone(),
                        type_of_field: field.ty.clone(),
                    }
                })
        }).next().or(None).unwrap_or(None)
    }

    pub fn get_qualifier_from_autowired(autowired_attr: Attribute) -> Option<String> {
        let autowired_tokens = autowired_attr.tokens.to_string();
        if autowired_attr.path.to_token_stream().to_string().clone().contains("autowired") {
            return autowired_tokens.strip_prefix("(")
                .map(|without_prefix| {
                    without_prefix.strip_suffix(")").or(None)
                })
                .map(|qualifier| qualifier
                    .map(|qualifier_found| {
                        println!("Found autowired attribute with tokens: {}.", qualifier_found);
                        String::from(qualifier_found)
                    })
                    .or(None)
                )
                .unwrap();
        }
        None
    }

    pub fn get_bean_type_from_factory_fn(attrs: Vec<Attribute>) -> Option<BeanType> {
        if attrs.iter().any(|attr| {
            attr.to_token_stream().to_string().contains("bean")
        }) {
            return attrs.iter().map(|attr| {
                if attr.to_token_stream().to_string().contains("singleton") {
                    let qualifier = attr.to_token_stream().to_string().clone();
                    println!("Found bean with qualifier: {}.", qualifier.clone());
                    return Some(BeanType::Singleton(BeanDefinition{ qualifier: None }));
                } else if attr.to_token_stream().to_string().contains("prototype") {
                    let qualifier = attr.to_token_stream().to_string().clone();
                    println!("Found bean with qualifier: {}.", qualifier.clone());
                    return Some(BeanType::Prototype(BeanDefinition{qualifier: None}));
                }
                None
            }).next().unwrap()
        }
        None
    }



}
