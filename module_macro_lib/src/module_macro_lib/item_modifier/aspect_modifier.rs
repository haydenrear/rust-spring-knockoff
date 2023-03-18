use crate::module_macro_lib::item_modifier::ItemModifier;

use std::ops::Deref;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned, ToTokens};
use rand::Rng;
use syn::{Block, FnArg, ImplItem, ImplItemMethod, Item, ItemImpl, parse, parse2, Pat, PatType, ReturnType, Stmt, Type};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use module_macro_shared::aspect::{AspectInfo, MethodAdviceChain};
use web_framework_shared::matcher::Matcher;
use crate::module_macro_lib::item_parser::item_impl_parser::{is_ignore_trait, matches_ignore_traits};
use crate::module_macro_lib::parse_container::ParseContainer;

use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;


pub struct AspectModifier;

impl ItemModifier for AspectModifier {
    fn modify_item(&self, parse_container: &mut ParseContainer, item: &mut Item, path_depth: Vec<String>) {
        match item {
            Item::Impl(item_impl) => {
                log_message!("Doing modification for {}.", SynHelper::get_str(&item_impl));
                let mut path_depth = path_depth.clone();
                path_depth.push(item_impl.self_ty.to_token_stream().to_string().clone());
                self.add_method_advice_aspect(
                    parse_container, item_impl,
                    &mut path_depth,
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
        if is_ignore_trait(&item_impl) {
            return;
        }
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
       method.sig.inputs.iter().flat_map(|i| {
            log_message!("Found fn_arg {}", SynHelper::get_str(i.clone()));
            match i {
                FnArg::Receiver(r) => {
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
        }).collect::<Vec<(Ident, Type)>>()
    }

    fn get_mutability(method: &ImplItemMethod) -> bool {
        method.sig.inputs.iter().any(|i| {
            log_message!("Found fn_arg {}", SynHelper::get_str(i.clone()));
            match i {
                FnArg::Receiver(r) => {
                    if r.mutability.is_some() {
                        return true;
                    }
                    false
                }
                FnArg::Typed(t) => {
                    false
                }
            }
        })
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

    fn parse_aspect_ordering(&self,
                             parse_container: &mut ParseContainer,
                             method: &mut ImplItemMethod,
                             path: &Vec<String>,
                             bean_id: &str) -> Vec<MethodAdviceAspectCodegen> {

        let mut advice = parse_container.aspects.aspects.iter()
            .flat_map(|p| &p.method_advice_aspects)
            .filter(|a| {
                let mut path = path.clone();
                let point_cut_matcher = path.join(".");
                log_message!("Checking if before advice {} and after advice {} matches {}.",
                    SynHelper::get_str(a.before_advice.clone().unwrap()),
                    SynHelper::get_str(a.after_advice.clone().unwrap()),
                    point_cut_matcher.clone()
                );
                if a.pointcut.pointcut_expr.matches(point_cut_matcher.as_str()) {
                    log_message!("Before advice {} and after advice {} matches {}!",
                        SynHelper::get_str(a.before_advice.clone().unwrap()),
                        SynHelper::get_str(a.after_advice.clone().unwrap()),
                        point_cut_matcher.clone()
                    );
                    return true;
                }
                false
            })
            .collect::<Vec<&MethodAdviceAspectCodegen>>();

        advice.sort();

        advice.iter()
            .map(|a| a.to_owned().to_owned())
            .collect::<Vec<MethodAdviceAspectCodegen>>()
    }

    fn parse_aspect(&self, parse_container: &mut ParseContainer, method: &mut ImplItemMethod, path: Vec<String>, args: Vec<(Ident, Type)>, bean_id: &str, return_type: Option<Type>) {
        log_message!("Doing aspect with {} aspects.", parse_container.aspects.aspects.len());

        let mut ordering = self.parse_aspect_ordering(parse_container, method, &path, bean_id);

        let chain = Self::parse_aspect_info(parse_container, method, args, bean_id, return_type, &mut ordering)
            .map(|mut a| Self::parse_advice_chain(&mut ordering, &mut a));

        chain.map(|chain| {
            parse_container.injectable_types_builder.get_mut(bean_id)
                .map(|b| b.aspect_info.push(chain));
        });

    }

    fn parse_aspect_info(
        parse_container: &mut ParseContainer, method: &mut ImplItemMethod, args: Vec<(Ident, Type)>, bean_id: &str, return_type: Option<Type>,
        codegen_items: &mut Vec<MethodAdviceAspectCodegen>) -> Option<AspectInfo> {
        if codegen_items.len() == 0 {
            return None;
        }

        Self::create_advice(
            parse_container, method, args, bean_id,
            return_type, &method.clone(),
            &mut codegen_items.remove(0),
        )
    }

    fn parse_advice_chain(items: &mut Vec<MethodAdviceAspectCodegen>, aspect_info: &mut AspectInfo) -> AspectInfo {
        aspect_info.advice_chain = items.iter_mut()
            .map(|mut m| Self::update_proceed_stmt_with_args(aspect_info, &mut m))
            .map(|v| MethodAdviceChain::new(&v))
            .collect();
        aspect_info.to_owned()
    }

    fn update_proceed_stmt_with_args(aspect_info: &AspectInfo, mut m: &mut MethodAdviceAspectCodegen) -> MethodAdviceAspectCodegen {
        let _ = AspectModifier::create_proceed_stmt_with_args(&aspect_info.method.as_ref().unwrap(), &mut m).ok()
            .map(|new_proceed| m.proceed_statement = Some(new_proceed));
        m.to_owned()
    }

    fn create_advice(parse_container: &mut ParseContainer, method: &mut ImplItemMethod, args: Vec<(Ident, Type)>, bean_id: &str,
                     return_type: Option<Type>, method_before: &ImplItemMethod, first: &mut MethodAdviceAspectCodegen
    ) -> Option<AspectInfo> {

        log_message!(
            "Adding before advice aspect: {}.",
            SynHelper::get_str(first.before_advice.clone().unwrap())
        );
        log_message!(
            "Adding after advice aspect: {}.",
            SynHelper::get_str(first.after_advice.clone().unwrap())
        );

        Self::add_advice_to_stmts(method, first);
        Self::rewrite_block_new_span(method);

        let return_type = return_type.clone();

        parse_container.injectable_types_builder.get_mut(bean_id)
            .map(|i| {
                AspectInfo {
                    method_advice_aspect: first.clone(),
                    method: Some(method_before.clone()),
                    args: args.clone(),
                    original_fn_logic: Some(method_before.block.clone()),
                    method_after: Some(method.clone()),
                    return_type,
                    mutable: Self::get_mutability(&method_before),
                    advice_chain: vec![],
                }
            })
    }

    fn rewrite_block_new_span(method: &mut ImplItemMethod) {
        let method_block_after = method.block.clone();
        let span = Span::call_site();
        let with_new_span = quote_spanned! {span=>
            #method_block_after
        }.into();
        let parsed = parse2::<Block>(with_new_span);
        method.block = parsed.unwrap();
    }

    fn add_advice_to_stmts(method: &mut ImplItemMethod, a: &mut MethodAdviceAspectCodegen) {
        let before = a.before_advice.clone();
        log_message!("Here is method block before: {}.", SynHelper::get_str(method.block.clone()));
        method.block.stmts.clear();

        Self::add_before_advice(method, before);
        Self::add_proceed_stmt(method, a);
        Self::add_after_advice(method, a);

        log_message!("Here is method block after: {}.", SynHelper::get_str(method.block.clone()));
    }

    fn add_proceed_stmt(method: &mut ImplItemMethod, a: &mut MethodAdviceAspectCodegen) {
        let stmt = Self::create_proceed_stmt_with_args(method, a);

        if stmt.is_err() {
            log_message!("Could not add proceed statement...");
        }
        stmt.ok().map(|p| {
            log_message!("Adding proceed statement created: {}.", SynHelper::get_str(&p));
            method.block.stmts.push(p.clone());
        }).or_else(|| {
            a.proceed_statement.as_ref().map(|p| method.block.stmts.push(p.clone()));
            None
        });
    }

    fn create_proceed_stmt_with_args(method: &ImplItemMethod, a: &mut MethodAdviceAspectCodegen) -> syn::Result<Stmt> {

        let args = method.sig.inputs.iter().flat_map(|f| {
            match f {
                FnArg::Receiver(r) => {
                    vec![]
                }
                FnArg::Typed(t) => {
                    vec![t.pat.deref().clone()]
                }
            }
        }).collect::<Vec<Pat>>();

        let ident = Self::get_proceed_ident(method);

        Self::create_new_proceed_stmt(&args, ident)
            .map(|proceed_stmt| {
                a.proceed_statement = Some(proceed_stmt.clone());
                proceed_stmt
            })
    }

    pub(crate) fn create_new_proceed_stmt<T: ToTokens>(args: &Vec<T>, ident: Ident) -> syn::Result<Stmt> {
        /// Important because every call to the function should contain all of the arguments of
        /// the original functions with all of the original names of the arguments to the functions.
        /// Then, at any level to the aspect all arguments are there, instead of calling the aspect
        /// at some n level deep with all the args and not having one of them there. Additionally,
        /// the move rules apply for this to work as long as the args continue to be passed in just as
        /// the first (this one)
        /// ** the first one is the only one that needs to be changed **
        let proceed = quote! {
            let found = self.#ident(#(#args),*);
        };

        log_message!("Adding proceed statement: {}.", SynHelper::get_str(&proceed));

        parse2::<Stmt>(proceed.into())
    }

    fn get_proceed_ident(method: &ImplItemMethod) -> Ident {
        let proceed_suffix= Self::create_proceed_suffix(&method.sig.ident);
        Self::create_proceed_ident_from_str(&proceed_suffix)
    }

    pub(crate) fn create_proceed_suffix(method_sig: &Ident) -> String {
        let mut ident = String::default();
        ident += method_sig.to_string().as_str();
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let x = rng.gen_range(b'A'..b'Z') as char;
            ident.push(x);
        }
        ident
    }

    pub(crate) fn create_proceed_ident_from_str(proceed_suffix: &String) -> Ident {
        let mut proceed_stmt = "proceed".to_string();
        proceed_stmt = proceed_stmt + proceed_suffix.as_str();
        proceed_stmt = proceed_stmt.replace(" ", "");
        let ident = Ident::new(proceed_stmt.as_str(), Span::call_site());
        ident
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
