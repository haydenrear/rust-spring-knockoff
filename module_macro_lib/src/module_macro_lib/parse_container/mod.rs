use std::any::{Any, TypeId};
use std::borrow::BorrowMut;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::collections::hash_map::Keys;
use std::fmt::{Debug, Formatter};
use std::iter::Filter;
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::slice::Iter;
use std::str::pattern::Pattern;
use std::sync::{Arc};
use proc_macro2::{Span, TokenStream};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf, TypeArray, ItemEnum, ReturnType, Stmt, Expr, Block, FnArg, PatType, Pat};
use syn::__private::{str, TokenStream as ts};
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    LitStr,
    Token,
    Ident,
    token::Paren,
};
use quote::{quote, format_ident, IdentFragment, ToTokens, quote_token, TokenStreamExt, quote_spanned};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use codegen_utils::syn_helper::SynHelper;
use crate::FieldAugmenterImpl;
use crate::module_macro_lib::bean_parser::{BeanDependencyParser, BeanParser};
use crate::module_macro_lib::context_builder::ContextBuilder;
use crate::module_macro_lib::fn_parser::FnParser;
use crate::module_macro_lib::initializer::ModuleMacroInitializer;
use crate::module_macro_lib::module_parser::parse_item;
use crate::module_macro_lib::module_tree::{Bean, Trait, Profile, DepType, BeanType, BeanDefinition, AutowiredField, AutowireType, InjectableTypeKey, ModulesFunctions, FunctionType, BeanDefinitionType, AspectInfo};
use crate::module_macro_lib::profile_tree::ProfileTree;
use crate::module_macro_lib::knockoff_context_builder::ApplicationContextGenerator;
use crate::module_macro_lib::util::ParseUtil;
use knockoff_logging::{initialize_log, use_logging, create_logger_expr};
use module_macro_codegen::aspect::{AspectParser, MethodAdviceAspectCodegen};
use web_framework_shared::matcher::Matcher;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;


#[derive(Default)]
pub struct ParseContainer {
    pub injectable_types_builder: HashMap<String, Bean>,
    pub injectable_types_map: ProfileTree,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<TypeId, ModulesFunctions>,
    pub profiles: Vec<Profile>,
    pub initializer: ModuleMacroInitializer,
    pub aspects: AspectParser
}

impl ParseContainer {

    /**
    Generate the token stream from the created ModuleContainer tree.
     **/
    pub fn build_to_token_stream(&mut self) -> TokenStream {
        ContextBuilder::build_token_stream(self)
    }

    pub fn build_injectable(&mut self) {
        self.set_build_dep_types();

        log_message!("{} is the number of injectable types.", &self.injectable_types_builder.values().len());
        self.injectable_types_map = ProfileTree::new(&mut self.injectable_types_builder);
        log_message!("{:?} is the debugged tree.", &self.injectable_types_map);
        log_message!("{} is the number of injectable types.", &self.injectable_types_map.injectable_types.values().len());
        log_message!("Here is the profile tree: ");
        self.injectable_types_map.injectable_types.values().flat_map(|b| {
                b.iter()
            })
            .for_each(|v| {
                log_message!("{:?} is the bean definition.", v.clone());
            })

    }

    pub fn set_build_dep_types(&mut self) {
        let keys = self.get_injectable_keys();
        let fns = self.fns.values().map(|fn_found| fn_found.fn_found.clone())
            .collect::<Vec<FunctionType>>();
        log_message!("{} is the number of injectable keys before.", keys.len());
        for id in keys.iter() {
            let mut removed = self.injectable_types_builder.remove(id).unwrap();
            let deps_set = BeanDependencyParser::add_dependencies(removed, &self.injectable_types_builder, &self.fns);
            self.injectable_types_builder.insert(id.clone().parse().unwrap(), deps_set);
        }
        for fn_type in fns.iter() {
            self.set_fn_type_dep(&fn_type);
        }
        log_message!("{} is the number of injectable keys after.", self.injectable_types_builder.len());
    }

    pub fn get_injectable_keys(&self) -> Vec<String> {
        self.injectable_types_builder.keys().map(|k| k.clone()).collect()
    }

    /**
    1. Make sure that there are no cyclic dependencies.
    2. Reorder so that the beans are added to the container in the correct order.
    **/
    pub fn is_valid_ordering_create(&self) -> Vec<String> {
        let mut already_processed = vec![];
        for i_type in self.injectable_types_builder.iter() {
            if !self.is_valid_ordering(&mut already_processed, i_type.1) {
                log_message!("Was not valid ordering!");
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
                log_message!("Could not find struct type with ident {}.", s.0.clone());
            })
    }

    /**
    Add the struct and the impl from the ItemImpl
     **/
    pub fn create_update_impl(&mut self, item_impl: &mut ItemImpl, path_depth: &mut Vec<String>) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        log_message!("Doing create update impl.");

        Self::add_path(path_depth, &item_impl);

        &mut self.injectable_types_builder.get_mut(&id)
            .map(|bean: &mut Bean| {
                bean.traits_impl.push(
                    AutowireType {
                        item_impl: item_impl.clone(),
                        profile: vec![],
                        path_depth: path_depth.clone()
                    }
                );
            })
            .or_else(|| {
                let mut impl_found = Bean {
                    struct_type: Some(item_impl.self_ty.deref().clone()),
                    struct_found: None,
                    traits_impl: vec![
                        AutowireType {
                            item_impl: item_impl.clone(),
                            profile: vec![],
                            path_depth: path_depth.clone()
                        }
                    ],
                    enum_found: None,
                    attr: vec![],
                    deps_map: vec![],
                    id: id.clone(),
                    path_depth: vec![],
                    profile: vec![],
                    ident: None,
                    fields: vec![],
                    bean_type: None,
                    mutable: SynHelper::get_attr_from_vec(&item_impl.attrs, vec!["mutable_bean"])
                        .map(|_| true)
                        .or(Some(false)).unwrap(),
                    aspect_info: None,
                };
                self.injectable_types_builder.insert(id.clone(), impl_found);
                None
            });

        log_message!("Adding method advice aspect now.");

        self.add_method_advice_aspect(item_impl, path_depth, &id);

    }

    fn add_path(path_depth: &mut Vec<String>, impl_found: &ItemImpl) {
        let mut trait_impl = vec![];
        impl_found.trait_.clone().map(|trait_found| {
            trait_impl.push(trait_found.1.to_token_stream().to_string());
        });
        trait_impl.push(impl_found.self_ty.to_token_stream().to_string().clone());
        path_depth.push(trait_impl.join("|"));
    }

    fn add_method_advice_aspect(&mut self, item_impl: &mut ItemImpl, path_depth: &mut Vec<String>, bean_id: &String) {
        log_message!("Adding method advice aspect to: {}", SynHelper::get_str(item_impl.clone()));
        item_impl.items.iter_mut()
            .for_each(|i| {
                match i {
                    ImplItem::Method(ref mut method) => {
                        log_message!("Found method {}", SynHelper::get_str(method.clone()));
                        let return_type = match &method.sig.output {
                            ReturnType::Default => {
                                None
                            }
                            ReturnType::Type(ty, ag) => {
                                Some(ag.deref().clone())
                            }
                        };
                        let args = method.sig.inputs.iter().flat_map(|i| {
                            log_message!("Found fn_arg {}", SynHelper::get_str(i.clone()));
                            match i {
                                FnArg::Receiver(_) => {
                                    vec![]
                                }
                                FnArg::Typed(t) => {
                                    log_message!("Found pat: {}", t.pat.to_token_stream().to_string().clone());
                                    match t.pat.deref().clone() {
                                        Pat::Ident(ident) => {
                                            log_message!("{} is the ident of the fn.", ident.ident.to_string().as_str());
                                            vec![(ident.ident, t.ty.deref().clone())]
                                        }
                                        _ => {
                                            vec![]
                                        }
                                    }
                                }
                            }
                        }).collect::<Vec<(Ident, Type)>>();

                        log_message!("Adding method advice aspect to: {}", SynHelper::get_str(method.clone()));
                        let mut next_path = path_depth.clone();
                        next_path.push(method.sig.ident.to_token_stream().to_string().clone());
                        log_message!("{} is the method before the method advice aspect.", SynHelper::get_str(method.block.clone()));
                        self.do_aspect(method, next_path, args, bean_id, return_type);
                        log_message!("{} is the method after the method advice aspect.", SynHelper::get_str(method.block.clone()));
                    }
                    _ => {}
                }
            });
    }

    fn do_aspect(&mut self, method: &mut ImplItemMethod, mut next_path: Vec<String>, args: Vec<(Ident, Type)>, bean_id: &String, return_type: Option<Type>) {
        log_message!("Doing aspect with {} aspects.", self.aspects.aspects.len());
        let method_before = method.clone();
        self.aspects.aspects.iter()
            .flat_map(|p| &p.method_advice_aspects)
            .filter(|a| {
                let point_cut_matcher = next_path.join(".");
                log_message!("Checking if before advice {} and after advice {} matches {}.",
                    SynHelper::get_str(a.before_advice.clone().unwrap()),
                    SynHelper::get_str(a.after_advice.clone().unwrap()),
                    point_cut_matcher.clone()
                );
                a.pointcut.pointcut_expr.matches(point_cut_matcher.as_str())
            })
            .for_each(|a| {

                log_message!("Adding before advice aspect: {}.", SynHelper::get_str(a.before_advice.clone().unwrap()));
                log_message!("Adding after advice aspect: {}.", SynHelper::get_str(a.after_advice.clone().unwrap()));

                Self::add_advice_to_stmts(method, &a);
                Self::rewrite_block_new_span(method);

                let return_type = return_type.clone();

                self.injectable_types_builder.get_mut(bean_id)
                    .map(|i| {
                        i.aspect_info = Some(AspectInfo {
                            method_advice_aspect: a.clone(),
                            method: Some(method_before.clone()),
                            args: args.clone(),
                            block: Some(method_before.block.clone()),
                            return_type
                        })
                    });
            });
    }

    fn rewrite_block_new_span(method: &mut ImplItemMethod) {
        let method_block_after = method.block.clone();
        let span = Span::call_site();
        let with_new_span = quote_spanned! {span=>
                                    #method_block_after
                                }.into();
        let parsed = parse::<Block>(with_new_span);
        method.block = parsed.unwrap();
    }

    fn add_advice_to_stmts(method: &mut ImplItemMethod, a: &MethodAdviceAspectCodegen) {
        let before = a.before_advice.clone();
        log_message!("Adding statements to method.");
        let stmts_to_check = method.block.stmts.clone();
        let proceed_stmt = stmts_to_check.iter()
            .filter(|p| p.to_token_stream().to_string().as_str().contains("proceed"))
            .next();
        method.block.stmts.clear();
        before.map(|mut before| {
            log_message!("Adding statements {} to method.", SynHelper::get_str(before.clone()));
            let mut before_stmts = before.stmts;
            for i in 0..before_stmts.len() {
                log_message!("Adding statement {} to method.", SynHelper::get_str(before_stmts.get(i).unwrap().clone()));
                method.block.stmts.insert(i, before_stmts.get(i).unwrap().to_owned())
            }
        });
        proceed_stmt.map(|p| method.block.stmts.push(p.clone()));
        a.after_advice.clone()
            .map(|after| after.stmts.iter()
                .for_each(|b| method.block.stmts.push(b.clone())));
    }

    pub fn add_item_struct(&mut self, item_impl: &mut ItemStruct, path_depth: Vec<String>) -> Option<String> {
        log_message!("adding type with name {}", item_impl.ident.clone().to_token_stream().to_string());
        log_message!("adding type with name {}", item_impl.to_token_stream().to_string().clone());

        self.injectable_types_builder.get_mut(&item_impl.ident.to_string().clone())
            .map(|struct_impl: &mut Bean| {
                struct_impl.struct_found = Some(item_impl.clone());
                struct_impl.ident =  Some(item_impl.ident.clone());
                struct_impl.fields = vec![item_impl.fields.clone()];
                struct_impl.bean_type = BeanParser::get_bean_type(&item_impl.attrs, None, Some(item_impl.ident.clone()));
                struct_impl.id = item_impl.ident.clone().to_string();
            })
            .or_else(|| {
                let mut impl_found = Bean {
                    struct_type: None,
                    struct_found: Some(item_impl.clone()),
                    traits_impl: vec![],
                    enum_found: None,
                    path_depth: path_depth.clone(),
                    attr: vec![],
                    deps_map: vec![],
                    id: item_impl.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(item_impl.ident.clone()),
                    fields: vec![item_impl.fields.clone()],
                    bean_type: BeanParser::get_bean_type(&item_impl.attrs, None, Some(item_impl.ident.clone())),
                    mutable: SynHelper::get_attr_from_vec(&item_impl.attrs, vec!["mutable_bean"])
                        .map(|_| true)
                        .or(Some(false)).unwrap(),
                    aspect_info: None,
                };
                self.injectable_types_builder.insert(item_impl.ident.to_string().clone(), impl_found);
                None
            });

        Some(item_impl.ident.to_string().clone())

    }

    pub fn add_item_enum(&mut self, enum_to_add: &mut ItemEnum, path_depth: Vec<String>) {
        log_message!("adding type with name {}", enum_to_add.ident.clone().to_token_stream().to_string());
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
                    path_depth,
                    struct_found: None,
                    traits_impl: vec![],
                    enum_found: Some(enum_to_add.clone()),
                    attr: vec![],
                    deps_map: vec![],
                    id: enum_to_add.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(enum_to_add.ident.clone()),
                    fields: enum_fields,
                    bean_type: BeanParser::get_bean_type(&enum_to_add.attrs, None, Some(enum_to_add.ident.clone())),
                    mutable: SynHelper::get_attr_from_vec(&enum_to_add.attrs, vec!["mutable_bean"])
                        .map(|_| true)
                        .or(Some(false)).unwrap(),
                    aspect_info: None,
                };
                self.injectable_types_builder.insert(enum_to_add.ident.to_string().clone(), impl_found);
                None
            });

    }

    pub fn create_update_trait(&mut self, trait_found: &mut ItemTrait) {
        if !self.traits.contains_key(&trait_found.ident.to_string().clone()) {
            self.traits.insert(trait_found.ident.to_string().clone(), Trait::new(trait_found.clone()));
        } else {
            log_message!("Contained trait already!");
        }
    }

    pub fn add_fn_to_dep_types(&mut self, item_fn: &mut ItemFn) {
        FnParser::to_fn_type(item_fn.clone())
            .map(|fn_found| {
                self.fns.insert(item_fn.clone().type_id().clone(), ModulesFunctions{ fn_found: fn_found.clone() });
            })
            .or_else(|| {
                log_message!("Could not set fn type for fn named: {}", SynHelper::get_str(item_fn.sig.ident.clone()).as_str());
                None
            });
    }

    fn set_fn_type_dep(&mut self, fn_found: &FunctionType) {
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
    }


    pub fn get_autowired_field_dep(field: Field) -> Option<AutowiredField> {
        SynHelper::get_attr_from_vec(&field.attrs, vec!["autowired"])
            .map(|autowired_field| {
                log_message!("Attempting to add autowired field for {}.", field.to_token_stream().to_string().as_str());
                SynHelper::get_attr_from_vec(&field.attrs, vec!["mutable_bean"])
                    .map(|mutable_field| {
                        log_message!("Adding mutable field and autowired field for {}.", field.to_token_stream().to_string().as_str());
                        AutowiredField{
                            qualifier: Some(autowired_field),
                            lazy: false,
                            field: field.clone(),
                            type_of_field: field.ty.clone(),
                            mutable: true,
                        }
                    })
                    .or(None)
            }).unwrap_or_else(|| {
                log_message!("Could not create autowired field of type {}.", field.ty.to_token_stream().to_string().clone());
                None
            })
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
