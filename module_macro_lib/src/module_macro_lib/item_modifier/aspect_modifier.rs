use crate::module_macro_lib::item_modifier::ItemModifier;

use std::ops::Deref;
use proc_macro2::{Ident, Span};
use quote::{quote_spanned, ToTokens};
use syn::{Block, FnArg, ImplItem, ImplItemMethod, Item, ItemImpl, parse, Pat, ReturnType, Type};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use web_framework_shared::matcher::Matcher;
use crate::module_macro_lib::parse_container::ParseContainer;

use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::module_tree::AspectInfo;


pub struct AspectModifier;

impl ItemModifier for AspectModifier {
    fn modify_item(&self, parse_container: &mut ParseContainer, item: &mut Item, path_depth: Vec<String>) {
        match item {
            Item::Impl(item_impl) => {
                log_message!("Doing modification for {}.", SynHelper::get_str(&item_impl));
                self.add_method_advice_aspect(
                    parse_container, item_impl,
                    &mut path_depth.clone(),
                    item_impl.self_ty.to_token_stream().to_string().as_str()
                );
            }
            _ => {}
        }
    }

    fn supports_item(&self, item: &Item) -> bool {
        match item {
            Item::Impl(_)  => {
                true
            }
            _ => {
                false
            }
        }
    }
}

impl AspectModifier {

    pub(crate) fn add_method_advice_aspect(&self, parse_container: &mut ParseContainer, item_impl: &mut ItemImpl, path_depth: &mut Vec<String>, bean_id: &str) {
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
                        self.parse_aspect(parse_container, method, next_path, args, bean_id, return_type);
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

    fn parse_aspect(&self, parse_container: &mut ParseContainer, method: &mut ImplItemMethod, path: Vec<String>, args: Vec<(Ident, Type)>, bean_id: &str, return_type: Option<Type>) {
        log_message!("Doing aspect with {} aspects.", parse_container.aspects.aspects.len());

        let method_before = method.clone();

        parse_container.aspects.aspects.iter()
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

                parse_container.injectable_types_builder.get_mut(bean_id)
                    .map(|i| {
                        i.aspect_info = Some(AspectInfo {
                            method_advice_aspect: a.clone(),
                            method: Some(method_before.clone()),
                            args: args.clone(),
                            block: Some(method_before.block.clone()),
                            method_after: Some(method.clone()),
                            return_type,
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
        log_message!("Here is method block before: {}.", SynHelper::get_str(method.block.clone()));
        method.block.stmts.clear();
        Self::add_before_advice(method, before);
        a.proceed_statement.as_ref().map(|p| method.block.stmts.push(p.clone()));
        Self::add_after_advice(method, a);
        log_message!("Here is method block after: {}.", SynHelper::get_str(method.block.clone()));
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
}
