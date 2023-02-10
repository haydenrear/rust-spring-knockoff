use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::collections::hash_map::Keys;
use std::fmt::{Debug, Formatter};
use std::iter::Filter;
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::slice::Iter;
use std::str::pattern::Pattern;
use std::sync::{Arc, Mutex};
use proc_macro2::TokenStream;
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
use crate::module_macro_lib::module_tree::{Bean, Trait, Profile, DepType, BeanType, BeanDefinition, AutowiredField, AutowireType, InjectableTypeKey, ModulesFunctions, FunctionType, BeanDefinitionType};
use crate::module_macro_lib::profile_tree::ProfileTree;
use crate::module_macro_lib::spring_knockoff_context::ApplicationContextGenerator;


#[derive(Default)]
pub struct ParseContainer {
    pub injectable_types_builder: HashMap<String, Bean>,
    pub injectable_types_map: ProfileTree,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<TypeId, ModulesFunctions>,
    pub profiles: Vec<Profile>
}

impl ParseContainer {
    /**
    Generate the token stream from the created ModuleContainer tree.
     **/
    pub fn to_token_stream(&mut self) -> proc_macro2::TokenStream {

        self.log_app_container_info();

        self.build_injectable();

        let mut token = quote! {};

        let mut profile_map = HashMap::new();

        self.injectable_types_map.injectable_types.iter()
            .flat_map(|bean_def_type_profile| bean_def_type_profile.1.iter()
                .map(move |bean_def_type| (bean_def_type_profile.0, bean_def_type))
            )
            .for_each(|bean_def_type_profile| {
                match bean_def_type_profile.1 {
                    BeanDefinitionType::Abstract { bean, dep_type } => {
                        Self::implement_abstract_autowire(&mut token, bean, bean_def_type_profile.0);
                        Self::insert_into_profile_map(&mut profile_map, bean_def_type_profile, bean);
                    }
                    BeanDefinitionType::Concrete { bean } => {
                        Self::implement_concrete_autowire(&mut token, bean, bean_def_type_profile.0);
                        Self::insert_into_profile_map(&mut profile_map, bean_def_type_profile, bean);
                    }
                }
            });

        self.finish_writing_factory(&mut token, profile_map);

        token

    }

    fn insert_into_profile_map(mut profile_map: &mut HashMap<Profile, Vec<Bean>>, bean_def_type_profile: (&Profile, &BeanDefinitionType), bean: &Bean) {
        if profile_map.contains_key(bean_def_type_profile.0) {
            profile_map.get_mut(bean_def_type_profile.0)
                .map(|beans| beans.push(bean.clone()));
        } else {
            let bean_vec = vec![bean.clone()];
            profile_map.insert(bean_def_type_profile.0.clone(), bean_vec);
        }
    }

    fn finish_writing_factory(&mut self, token: &mut TokenStream, beans: HashMap<Profile, Vec<Bean>>) {

        beans.iter().for_each(|profile_type| {
            println!("Creating bean factory for profile type: {}.", profile_type.0.profile.clone());
            let listable_bean_factory = ApplicationContextGenerator::new_listable_bean_factory(
                profile_type.1.clone(),
                profile_type.0.clone()
            );

            token.extend(listable_bean_factory.into_iter());

            token.append_all(ApplicationContextGenerator::finish_abstract_factory(vec![profile_type.0.profile.clone()]));
        })

    }

    fn implement_abstract_autowire(mut token: &mut TokenStream, token_type: &Bean, profile: &Profile) {

        let (field_types, identifiers) = Self::get_field_ids(token_type);

        if token_type.struct_type.is_some() {
            let struct_type = token_type.struct_type.clone()
                .unwrap();
            Self::implement_abstract_code(&mut token, &field_types, &identifiers, &struct_type);
        } else if token_type.ident.is_some() {
            let struct_type = token_type.ident.clone()
                .unwrap();
            Self::implement_abstract_code(&mut token, &field_types, &identifiers, &struct_type);
        }
    }

    fn implement_concrete_autowire(mut token: &mut TokenStream, token_type: &Bean, profile: &Profile) {

        let (field_types, identifiers) = Self::get_field_ids(token_type);

        if token_type.struct_type.is_some() {
            let struct_type = token_type.struct_type.clone()
                .unwrap();
            Self::implement_autowire_code(&mut token, &field_types, &identifiers, &struct_type);
        } else if token_type.ident.is_some() {
            let struct_type = token_type.ident.clone()
                .unwrap();
            Self::implement_autowire_code(&mut token, &field_types, &identifiers, &struct_type);
        }
    }

    fn get_field_ids(token_type: &Bean) -> (Vec<Type>, Vec<Ident>) {
        let field_types = token_type.deps_map
            .clone().iter()
            .map(|d| d.bean_info.type_of_field.clone())
            .collect::<Vec<Type>>();

        let identifiers = token_type.deps_map
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
        (field_types, identifiers)
    }

    fn implement_autowire_code<T: ToTokens>(token: &mut TokenStream, field_types: &Vec<Type>, identifiers: &Vec<Ident>, struct_type: &T) {
        println!("Implementing container for {}.", struct_type.to_token_stream().to_string().clone());

        let this_struct_impl = ApplicationContextGenerator::gen_autowire_code_gen_concrete(
            &field_types, &identifiers, &struct_type
        );

        token.append_all(this_struct_impl);
    }

    fn implement_abstract_code<T: ToTokens>(token: &mut TokenStream, field_types: &Vec<Type>, identifiers: &Vec<Ident>, struct_type: &T) {
        println!("Implementing container for {}.", struct_type.to_token_stream().to_string().clone());

        let this_struct_impl = ApplicationContextGenerator::gen_autowire_code_gen_abstract(
            &field_types, &identifiers, &struct_type
        );

        token.append_all(this_struct_impl);
    }

    pub fn build_concrete_types(map: &HashMap<String, Bean>) -> HashMap<InjectableTypeKey, Bean> {
        let mut return_map = HashMap::new();
        for i_type in map.iter() {
            return_map.insert(InjectableTypeKey {
                underlying_type: i_type.0.clone(),
                impl_type: None,
                profile: i_type.1.profile.clone()
            }, i_type.1.clone());
        }
        return_map
    }

    pub fn build_injectable(&mut self) {
        self.injectable_types_map = ProfileTree::new(&self.injectable_types_builder);
        println!("{:?} is the debugged tree.", &self.injectable_types_map);
        println!("{} is the number of injectable types.", &self.injectable_types_builder.len());
    }

    /**
    1. Make sure that there are no cyclic dependencies.
    2. Reorder so that the beans are added to the container in the correct order.
    **/
    pub fn is_valid_ordering_create(&self) -> Vec<String> {
        let mut already_processed = vec![];
        for i_type in self.injectable_types_builder.iter() {
            if !self.is_valid_ordering(&mut already_processed, i_type.1) {
                println!("Was not valid ordering!");
                return vec![];
            }
        }
        already_processed
    }

    pub fn is_valid_ordering(&self, already_processed: &mut Vec<String>, dep: &Bean) -> bool {
        already_processed.push(dep.id.clone());
        for dep_impl in &dep.deps_map {
            let next_id = ParseContainer::get_identifier(dep_impl);
            if already_processed.contains(&next_id) {
                continue;
            }
            if !self.injectable_types_builder.get(&next_id)
                .map(|next| {
                    return self.is_valid_ordering(already_processed, next);
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
                dep_type.bean_info.type_of_field.to_token_stream().to_string()
            }
            Some(qual) => {
                qual.clone()
            }
        }
    }

    pub fn log_app_container_info(&self) {
        self.injectable_types_builder.iter().filter(|&s| s.1.struct_found.is_none())
            .for_each(|s| {
                println!("Could not find struct type with ident {}.", s.0.clone());
            })
    }

    /**
    Add the struct and the impl from the ItemImpl
     **/
    pub fn create_update_impl(&mut self, item_impl: &mut ItemImpl) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        &mut self.injectable_types_builder.get_mut(&id)
            .map(|struct_impl: &mut Bean| {
                struct_impl.traits_impl.push(AutowireType { item_impl: item_impl.clone(), profile: vec![] });
            })
            .or_else(|| {
                let mut impl_found = Bean {
                    struct_type: Some(item_impl.self_ty.deref().clone()),
                    struct_found: None,
                    traits_impl: vec![AutowireType { item_impl: item_impl.clone(), profile: vec![] }],
                    enum_found: None,
                    attr: vec![],
                    deps_map: vec![],
                    id: id.clone(),
                    profile: vec![],
                    ident: None,
                    fields: vec![],
                    bean_type: None
                };
                self.injectable_types_builder.insert(id.clone(), impl_found);
                None
            });

        self.set_deps_safe(id.as_str());
    }

    pub fn add_item_struct(&mut self, item_impl: &mut ItemStruct) -> Option<String> {
        println!("adding type with name {}", item_impl.ident.clone().to_token_stream().to_string());
        println!("adding type with name {}", item_impl.to_token_stream().to_string().clone());

        self.injectable_types_builder.get_mut(&item_impl.ident.to_string().clone())
            .map(|struct_impl: &mut Bean| {
                struct_impl.struct_found = Some(item_impl.clone());
                struct_impl.ident =  Some(item_impl.ident.clone());
                struct_impl.fields = vec![item_impl.fields.clone()];
                struct_impl.bean_type = ParseContainer::get_bean_type(&item_impl.attrs, None, Some(item_impl.ident.clone()));
                struct_impl.id = item_impl.ident.clone().to_string();
            })
            .or_else(|| {
                let mut impl_found = Bean {
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
                    bean_type: ParseContainer::get_bean_type(&item_impl.attrs, None, Some(item_impl.ident.clone()))
                };
                self.injectable_types_builder.insert(item_impl.ident.to_string().clone(), impl_found);
                None
            });

        self.set_deps_safe(item_impl.ident.to_string().as_str());
        Some(item_impl.ident.to_string().clone())

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

    pub fn add_item_enum(&mut self, enum_to_add: &mut ItemEnum) {
        println!("adding type with name {}", enum_to_add.ident.clone().to_token_stream().to_string());
        &mut self.injectable_types_builder.get_mut(&enum_to_add.ident.to_string().clone())
            .map(|struct_impl: &mut Bean| {
                struct_impl.enum_found = Some(enum_to_add.clone());
            })
            .or_else(|| {
                let enum_fields = enum_to_add.variants.iter()
                    .map(|variant| variant.fields.clone())
                    .collect::<Vec<Fields>>();
                let mut impl_found = Bean {
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
                    bean_type: ParseContainer::get_bean_type(&enum_to_add.attrs, None, Some(enum_to_add.ident.clone()))
                };
                self.injectable_types_builder.insert(enum_to_add.ident.to_string().clone(), impl_found);
                None
            });

        self.set_deps_safe(enum_to_add.ident.to_string().as_str());
    }

    fn set_deps_safe(&mut self, id: &str) {
        let mut removed = self.injectable_types_builder.remove(id).unwrap();
        let deps_set = self.add_dependencies(removed);
        self.injectable_types_builder.insert(id.clone().parse().unwrap(), deps_set);
    }

    pub fn create_update_trait(&mut self, trait_found: &mut ItemTrait) {
        if !self.traits.contains_key(&trait_found.ident.to_string().clone()) {
            self.traits.insert(trait_found.ident.to_string().clone(), Trait::new(trait_found.clone()));
        } else {
            println!("Contained trait already!");
        }
    }

    pub fn add_dependencies(&self, mut bean: Bean) -> Bean {
        for fields in bean.fields.clone().iter() {
            match fields.clone() {
                Fields::Named(fields_named) => {
                    for field in fields_named.named.iter() {
                        field.clone().ident.map(|ident: Ident| {
                            println!("found field {}.", ident.to_string().clone());
                        });
                        println!("{} is the field type!", field.ty.to_token_stream().clone());
                        bean = self.match_ty_add_dep(
                            bean,
                            None,
                            None,
                            field.clone()
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
        &self,
        mut dep_impl: Bean,
        lifetime: Option<Lifetime>,
        array_type: Option<TypeArray>,
        field: Field,
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
                        dep_impl = self.add_type_dep(dep_impl, autowired, lifetime, Some(arr));
                    }
                    Type::Path(path) => {
                        println!("Adding type path.");
                        //TODO: extension point for lazy
                        dep_impl = self.add_type_dep(dep_impl, autowired, lifetime, array_type);
                    }
                    Type::Reference(reference_found) => {
                        let ref_type = reference_found.elem.clone();
                        println!("{} is the ref type", ref_type.to_token_stream());
                        dep_impl = self.add_type_dep(dep_impl, autowired, reference_found.lifetime, array_type);
                    }
                    _ => {
                        dep_impl = self.add_type_dep(dep_impl, autowired, lifetime, array_type)
                    }
                };
                dep_impl
            }
        }
    }

    pub fn add_fn_to_dep_types(&mut self, item_fn: &mut ItemFn) {
        ParseContainer::get_fn_type(item_fn.clone())
            .map(|fn_found| {
                self.fns.insert(item_fn.clone().type_id().clone(), ModulesFunctions{ fn_found: fn_found.clone() });
                for i_type in self.injectable_types_builder.iter_mut() {
                    for dep_type in i_type.1.deps_map.iter_mut() {
                        if dep_type.bean_type.is_none() {
                            match &fn_found {
                                FunctionType::Singleton(fn_type, qualifier, type_found) => {
                                    dep_type.bean_type = Some(
                                        BeanType::Singleton(
                                            BeanDefinition {
                                                qualifier: qualifier.clone(),
                                                bean_type_type: type_found.clone(),
                                                bean_type_ident: None
                                            },
                                            Some(fn_found.clone()))
                                    );
                                }
                                FunctionType::Prototype(fn_type, qualifier, type_found) => {
                                    dep_type.bean_type = Some(
                                        BeanType::Prototype(
                                        BeanDefinition {
                                            qualifier: qualifier.clone(),
                                            bean_type_type: type_found.clone(),
                                            bean_type_ident: None
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
        &self, mut dep_impl: Bean, field_to_add: AutowiredField, lifetime: Option<Lifetime>, array_type: Option<TypeArray>
    ) -> Bean
    {
        println!("Adding dependency for {}.", dep_impl.id.clone());
        let type_dep = &field_to_add.field.clone().ty.to_token_stream().to_string();
        let contains_key = self.injectable_types_builder.contains_key(type_dep);
        let struct_exists = self.injectable_types_builder.get(&field_to_add.field.clone().ty.to_token_stream().to_string()).is_some();
        let autowired_qualifier = field_to_add.clone().qualifier.or(Some(field_to_add.type_of_field.to_token_stream().to_string().clone()));
        if autowired_qualifier.is_some() && contains_key && struct_exists {

            dep_impl.ident.clone().map(|ident| {
                println!("Adding dependency with id {} to struct_impl of name {}", dep_impl.id.clone(), ident.to_string().clone());
            }).or_else(|| {
                println!("Could not find ident for {}.", dep_impl.id.clone());
                None
            });

            let bean_type = self.get_fn_for_qualifier(
                autowired_qualifier.clone(),
                Some(field_to_add.type_of_field.clone())
            ).map(|fn_type| {
                ParseContainer::get_bean_type_from_qual(autowired_qualifier, None, fn_type)
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

    fn get_fn_for_qualifier(&self, qualifier: Option<String>, type_of: Option<Type>) -> Option<FunctionType> {
        for module_fn_entry in &self.fns {
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

    fn get_fn_type(fn_found: ItemFn) -> Option<FunctionType> {
        Self::filter_singleton_prototype(&fn_found.attrs)
            .iter()
            .flat_map(|attr| {
                    match fn_found.sig.output.clone() {
                        ReturnType::Default => {
                            if attr.to_token_stream().to_string().contains("singleton") {
                                Some(FunctionType::Singleton(
                                    fn_found.clone(),
                                    ParseContainer::strip_value(attr.path.to_token_stream().to_string().as_str())
                                        .map(|qual| String::from(qual)),
                                    None
                                ))
                            } else if attr.to_token_stream().to_string().contains("prototype") {
                                Some(FunctionType::Prototype(
                                    fn_found.clone(),
                                    ParseContainer::strip_value(attr.path.to_token_stream().to_string().as_str())
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
                                    ParseContainer::strip_value(attr.path.to_token_stream().to_string().as_str())
                                        .map(|qual| String::from(qual)),
                                    Some(ty.deref().clone())
                                ))
                            } else if attr.to_token_stream().to_string().contains("prototype") {
                                Some(FunctionType::Prototype(
                                    fn_found.clone(),
                                    ParseContainer::strip_value(attr.path.to_token_stream().to_string().as_str())
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

    fn get_prototype_or_singleton(attr: &Vec<Attribute>, bean_type: Option<Type>, bean_type_ident: Option<Ident>) -> Option<BeanType> {
        Self::filter_singleton_prototype(attr)
            .and_then(|s| {
                let qualifier = Self::strip_value_attr(s);

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

    fn filter_singleton_prototype(attr: &Vec<Attribute>) -> Option<&Attribute> {
        attr.into_iter()
            .filter(|&attr| {
                let attr_name = attr.to_token_stream().to_string();
                println!("Checking attribute: {} for fn.", attr_name.clone());
                attr_name.contains("singleton") || attr_name.contains("prototype")
            }).next()
    }

    pub fn get_autowired_field_dep(attrs: Vec<Attribute>, field: Field) -> Option<AutowiredField> {
        println!("Checking attributes for field {}.", field.to_token_stream().to_string().clone());
        attrs.iter().map(|attr| {
            println!("Checking attribute: {} for field.", attr.to_token_stream().to_string().clone());
            let mut autowired_field = AutowiredField{
                qualifier: None,
                lazy: false,
                field: field.clone(),
                type_of_field: field.ty.clone(),
            };
            ParseContainer::get_qualifier_from_autowired(attr.clone())
                .map(|autowired_value| {
                    autowired_field.qualifier = Some(autowired_value);
                });
            Some(autowired_field)
        }).next().unwrap_or_else(|| {
            println!("Could not create autowired field of type {}.", field.ty.to_token_stream().to_string().clone());
            None
        })
    }

    pub fn strip_value(value: &str) -> Option<String> {
        println!("Stripping prefix {}.", value);
        value.strip_prefix("#[singleton(")
            .map(|without_singleton| {
                println!("{} is without singleton", without_singleton);
                without_singleton.strip_prefix("#[prototype(")
                    .or(Some(without_singleton)).unwrap()
            })
            .map(|without_prefix| {
                println!("{} is without singleton and prototype", without_prefix);
                without_prefix.strip_suffix(")]")
                    .map(|str| String::from(str))
                    .or(None)
            }).unwrap_or(None).map(|value_found| {
                println!("Found bean with qualifier {}.", value_found.as_str());
                value_found
            })
    }

    pub fn strip_value_attr(attr: &Attribute) -> Option<String> {
        Self::strip_value(attr.to_token_stream().to_string().as_str())
    }

    pub fn get_qualifier_from_autowired(autowired_attr: Attribute) -> Option<String> {
        if autowired_attr.path.to_token_stream().to_string().clone().contains("autowired") {
            return ParseContainer::strip_value(autowired_attr.path.to_token_stream().to_string().as_str());
        }
        None
    }

    pub fn get_bean_type_from_factory_fn(attrs: Vec<Attribute>, module_fn: ModulesFunctions) -> Option<BeanType> {
        if attrs.iter().any(|attr| {
            let attr_str = attr.to_token_stream().to_string();
            attr_str.contains("bean") || attr_str.contains("singleton") || attr_str.contains("prototype")
        }) {
            return attrs.iter().flat_map(|attr| {
                let qualifier = ParseContainer::strip_value(attr.path.to_token_stream().to_string().as_str());
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
