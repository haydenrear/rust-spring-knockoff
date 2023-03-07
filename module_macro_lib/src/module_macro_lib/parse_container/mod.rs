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
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
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

    pub(crate) fn add_method_advice_aspect(&mut self, item_impl: &mut ItemImpl, path_depth: &mut Vec<String>, bean_id: &String) {
        log_message!("Adding method advice aspect to: {}", SynHelper::get_str(item_impl.clone()));
        item_impl.items.iter_mut()
            .for_each(|i| {
                match i {
                    ImplItem::Method(ref mut method) => {
                        log_message!("Found method {}", SynHelper::get_str(method.clone()));
                        let return_type = Self::get_return_type(&method);
                        let args = Self::get_args_info(method);
                        log_message!("Adding method advice aspect to: {}", SynHelper::get_str(method.clone()));
                        let mut next_path = path_depth.clone();
                        next_path.push(method.sig.ident.to_token_stream().to_string().clone());
                        log_message!("{} is the method before the method advice aspect.", SynHelper::get_str(method.block.clone()));
                        self.parse_aspect(method, next_path, args, bean_id, return_type);
                        log_message!("{} is the method after the method advice aspect.", SynHelper::get_str(method.block.clone()));
                    }
                    _ => {}
                }
            });
    }

    fn get_args_info(method: &mut ImplItemMethod) -> Vec<(Ident, Type)> {
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
        args
    }

    fn get_return_type(method: &&mut ImplItemMethod) -> Option<Type> {
        let return_type = match &method.sig.output {
            ReturnType::Default => {
                None
            }
            ReturnType::Type(ty, ag) => {
                Some(ag.deref().clone())
            }
        };
        return_type
    }

    fn parse_aspect(&mut self, method: &mut ImplItemMethod, path: Vec<String>, args: Vec<(Ident, Type)>, bean_id: &String, return_type: Option<Type>) {
        log_message!("Doing aspect with {} aspects.", self.aspects.aspects.len());

        let method_before = method.clone();

        self.aspects.aspects.iter()
            .flat_map(|p| &p.method_advice_aspects)
            .filter(|a| {
                let point_cut_matcher = path.join(".");
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
                            method_after: Some(method.clone()),
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
        method.block.stmts.clear();
        log_message!("Here is method block before: {}.", SynHelper::get_str(method.block.clone()));
        Self::add_before_advice(method, before);
        Self::add_after_advice(method, a);
    }

    fn add_after_advice(method: &mut ImplItemMethod, a: &MethodAdviceAspectCodegen) {
        a.after_advice.clone()
            .map(|after| after.stmts.iter()
                .for_each(|b| method.block.stmts.push(b.clone())));
    }

    fn add_before_advice(method: &mut ImplItemMethod, before: Option<Block>) {
        before.map(|mut before| {
            log_message!("Adding statements {} to method.", SynHelper::get_str(before.clone()));
            let mut before_stmts = before.stmts;
            for i in 0..before_stmts.len() {
                log_message!("Adding statement {} to method.", SynHelper::get_str(before_stmts.get(i).unwrap().clone()));
                method.block.stmts.insert(i, before_stmts.get(i).unwrap().to_owned())
            }
            log_message!("Here are statements after: {}", SynHelper::get_str(method.block.clone()));
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
