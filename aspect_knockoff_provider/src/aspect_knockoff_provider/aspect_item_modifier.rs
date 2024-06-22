use std::any::Any;
use std::collections::HashMap;
use module_macro_shared::parse_container::{MetadataItem, MetadataItemId, ParseContainer};
use syn::{Block, FnArg, ImplItem, ImplItemMethod, Item, ItemImpl, parse2, Pat, ReturnType, Stmt, Type};
use proc_macro2::{Ident, Span};
use codegen_utils::syn_helper::SynHelper;
use quote::{quote, quote_spanned, ToTokens};
use std::ops::Deref;
use std::sync::Arc;
use rand::Rng;
use collection_util::add_to_multi_value;
use web_framework_shared::matcher::Matcher;
use crate::aspect_knockoff_provider;
use crate::aspect_knockoff_provider::aspect_parse_provider::MethodAdviceAspectCodegen;
use crate::aspect_knockoff_provider::{AspectInfo, MethodAdviceChain};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use module_macro_shared::item_impl_parser::ItemImplParser;
use optional::FlatMapOptional;
use crate::logger_lazy;
import_logger!("aspect_knockoff_provider.rs");

#[derive(Default, Clone)]
pub struct AspectParser;

/// Adds the aspects to the ParseContainer, and updates the functions to wrap.
impl AspectParser {

    pub fn new() -> Self {
        Self {}
    }

    /// This runs after the parse provider, which means that the aspects from the program have already
    /// been loaded into the ParseContainer. So now the item can be modified to add the aspect.
    pub fn modify_item(parse_container: &mut ParseContainer,
                       item: &mut Item, path_depth: Vec<String>) {
        match item {
            Item::Impl(item_impl) => {
                info!("Doing modification for {}.", SynHelper::get_str(&item_impl));
                let mut path_depth = path_depth.clone();
                path_depth.push(item_impl.self_ty.deref().to_token_stream().to_string().clone());
                Self::add_method_advice_aspect(
                    parse_container, item_impl,
                    &mut path_depth, item_impl.self_ty.deref().to_token_stream().to_string().as_str(),
                );
            }
            _ => {}
        };
    }

    pub fn supports(item: &Item) -> bool {
        match item {
            Item::Fn(item_fn) => {
                let is_aspect = MethodAdviceAspectCodegen::is_aspect(&item_fn.attrs);
                info!("Testing if {:?} is decorated with aspect.", SynHelper::get_str(item));
                is_aspect
            }
            _ => {
                false
            }
        }
    }

    pub fn supports_item(item: &Item) -> bool {
        match item {
            Item::Impl(_)  => {
                true
            }
            _ => {
                false
            }
        }
    }

    pub fn add_method_advice_aspect(parse_container: &mut ParseContainer,
                                    item_impl: &mut ItemImpl,
                                    path_depth: &mut Vec<String>,
                                    bean_id: &str) {
        if aspect_knockoff_provider::is_ignore_trait(&item_impl) {
            return;
        }
        item_impl.items.iter_mut()
            .for_each(|i| {
                match i {
                    ImplItem::Method(ref mut method) => {
                        info!("Found method {}", SynHelper::get_str(method.clone()));
                        let return_type = Self::get_return_type(&method);
                        let args = Self::get_args_info(method);
                        info!("Adding method advice aspect to: {}", SynHelper::get_str(method.clone()));
                        let mut next_path = path_depth.clone();
                        next_path.push(method.sig.ident.to_token_stream().to_string().clone());
                        info!("{} is the method before the method advice aspect.", SynHelper::get_str(method.block.clone()));
                        Self::parse_aspect(parse_container, method, next_path, args, bean_id, return_type);
                        info!("{} is the method after the method advice aspect.", SynHelper::get_str(method.block.clone()));
                    }
                    _ => {}
                }
            });
    }

    fn get_args_info(method: &mut ImplItemMethod) -> Vec<(Ident, Type)> {
        method.sig.inputs.iter().flat_map(|i| {
            info!("Found fn_arg {}", SynHelper::get_str(i.clone()));
            match i {
                FnArg::Receiver(r) => {
                    vec![]
                }
                FnArg::Typed(t) => {
                    info!("Found pat: {}", t.pat.to_token_stream().to_string().clone());
                    SynHelper::get_fn_arg_ident_type(t)
                        .map(|t| vec![t])
                        .or(Some(vec![]))
                        .unwrap()
                }
            }
        }).collect::<Vec<(Ident, Type)>>()
    }

    fn get_mutability(method: &ImplItemMethod) -> bool {
        method.sig.inputs.iter().any(|i| {
            info!("Found fn_arg {}", SynHelper::get_str(i.clone()));
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

    fn parse_aspect_ordering(parse_container: &mut ParseContainer,
                             path: &Vec<String>) -> Vec<MethodAdviceAspectCodegen> {
        info!("Parsing aspect ordering.");
        let retrieved = Self::retrieve_to_remove(parse_container);

        if retrieved.is_none() {
            info!("Did not retrieve anything.");
            return vec![];
        }

        let removed_value = retrieved.unwrap();

        let aspects = removed_value
            .into_iter()
            .flat_map(|values|
                MethodAdviceAspectCodegen::parse_values(&mut Some(values))
                    .cloned().into_iter()
            )
            .collect::<Vec<MethodAdviceAspectCodegen>>();

        info!("Found {} aspects.", aspects.len());

        let mut advice = aspects.clone().into_iter()
            .filter(|a| {
                let mut path = path.clone();
                let point_cut_matcher = path.join(".");
                info!("{:?} is the next advice", a);
                info!("Checking if before advice {} and after advice {} matches {}.",
                    SynHelper::get_str(a.before_advice.clone().unwrap()),
                    SynHelper::get_str(a.after_advice.clone().unwrap()),
                    point_cut_matcher.clone()
                );
                info!("Checking if {:?} matches {:?}", a.pointcut.pointcut_expr, point_cut_matcher.as_str());
                if a.pointcut.pointcut_expr.matches(point_cut_matcher.as_str()) {
                    info!("Before advice {} and after advice {} matches {}!",
                        SynHelper::get_str(a.before_advice.clone().unwrap()),
                        SynHelper::get_str(a.after_advice.clone().unwrap()),
                        point_cut_matcher.clone()
                    );
                    return true;
                }
                false
            })
            .collect::<Vec<MethodAdviceAspectCodegen>>();

        advice.sort();

        let adv = advice.into_iter()
            .map(|a| a.to_owned())
            .collect::<Vec<MethodAdviceAspectCodegen>>();

        let metadata = MetadataItemId::new("".to_string(), "MethodAdviceAspectCodegen".to_string());
        aspects.into_iter().for_each(|v| {
            add_to_multi_value(&mut parse_container.provided_items,
                               Box::new(v) as Box<dyn MetadataItem>, metadata.clone());
        });

        adv
    }

    fn retrieve_to_remove(parse_container: &mut ParseContainer) -> Option<Vec<Box<dyn MetadataItem>>> {
        info!("Retrieving method advice aspect codegen.");
        let metadata: Vec<MetadataItemId> = parse_container.provided_items.keys()
            .filter(|k| k.metadata_item_type_id == "MethodAdviceAspectCodegen")
            .peekable()
            .map(|f| {
                info!("Found method advice aspect: {:?}", f);
                f
            })
            .map(|k| k.clone())
            .collect();

        assert!(metadata.len() <= 1);
        if metadata.len() == 0 {
            return None
        }
        let k = metadata.iter().next().unwrap();
        let removed_value = parse_container.provided_items.remove(k);
        if removed_value.as_ref().is_some() {
            info!("Found {} number of values.", removed_value.as_ref().unwrap().len());
        }
        removed_value
    }

    fn parse_aspect(parse_container: &mut ParseContainer,
                    method: &mut ImplItemMethod,
                    path: Vec<String>,
                    args: Vec<(Ident, Type)>,
                    bean_id: &str, return_type: Option<Type>) {
        info!("Parsing aspect: {:?}", SynHelper::get_str(&method));
        let mut ordering = Self::parse_aspect_ordering(parse_container, &path);

        let chain = Self::parse_aspect_info(method, args, bean_id,
                                            return_type, &mut ordering)
            .map(|mut a| {
                info!("Found aspect info to be added: {:?} before aspect parsing advice chain.", a);
                Self::parse_advice_chain(&mut ordering, &mut a)
            })
            .map(|aspect_info| {
                info!("Found aspect info to be added: {:?}", aspect_info);
                aspect_info
            })
            .or_else(|| {
                info!("No method advice aspect to be found for {} and method {:?}.",
                    bean_id, method.to_token_stream().to_string().as_str());
                None
            });

        chain.map(|chain| {
            info!("Inserting aspect info...");
            let metadata_id = MetadataItemId::new(bean_id.to_string(),
                                                  "AspectInfo".to_string());
            add_to_multi_value(&mut parse_container.provided_items,
                               Box::new(chain), metadata_id
            );
        });

    }

    fn parse_aspect_info(
        method: &mut ImplItemMethod, args: Vec<(Ident, Type)>,
        bean_id: &str, return_type: Option<Type>,
        codegen_items: &mut Vec<MethodAdviceAspectCodegen>
    ) -> Option<AspectInfo> {
        if codegen_items.len() == 0 {
            info!("No codegen items found.");
            return None;
        }

        Self::create_advice(method, args, bean_id,
                            return_type, &method.clone(),
                            &mut codegen_items.remove(0),
        )
    }

    fn parse_advice_chain(items: &mut Vec<MethodAdviceAspectCodegen>, aspect_info: &mut AspectInfo) -> AspectInfo {
        info!("Parsing aspect info advice chain.");
        aspect_info.advice_chain = items.iter_mut()
            .map(|mut m| Self::update_proceed_stmt_with_args(aspect_info, &mut m))
            .map(|v| MethodAdviceChain::new(&v))
            .collect();
        aspect_info.to_owned()
    }

    fn update_proceed_stmt_with_args(aspect_info: &AspectInfo, mut m: &mut MethodAdviceAspectCodegen) -> MethodAdviceAspectCodegen {
        info!("Updating proceed stmt: {:?}", aspect_info);
        let _ = Self::create_proceed_stmt_with_args(&aspect_info.method.as_ref().unwrap(), &mut m).ok()
            .map(|new_proceed| m.proceed_statement = Some(new_proceed));
        m.to_owned()
    }

    fn create_advice(method: &mut ImplItemMethod, args: Vec<(Ident, Type)>, bean_id: &str,
                     return_type: Option<Type>, method_before: &ImplItemMethod, first: &mut MethodAdviceAspectCodegen
    ) -> Option<AspectInfo> {

        info!("Creating advice: {:?}", first);
        info!(
            "Adding before advice aspect: {}.",
            SynHelper::get_str(first.before_advice.clone().unwrap())
        );
        info!(
            "Adding after advice aspect: {}.",
            SynHelper::get_str(first.after_advice.clone().unwrap())
        );

        Self::add_advice_to_stmts(method, first);
        Self::rewrite_block_new_span(method);

        let return_type = return_type.clone();

        Some(AspectInfo {
            method_advice_aspect: first.clone(),
            method: Some(method_before.clone()),
            args: args.clone(),
            original_fn_logic: Some(method_before.block.clone()),
            method_after: Some(method.clone()),
            return_type,
            mutable: Self::get_mutability(&method_before),
            advice_chain: vec![],
            ty: bean_id.to_string()
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
        info!("Here is method block before: {}.", SynHelper::get_str(method.block.clone()));
        method.block.stmts.clear();

        Self::add_before_advice(method, before);
        Self::add_proceed_stmt(method, a);
        Self::add_after_advice(method, a);

        info!("Here is method block after: {}.", SynHelper::get_str(method.block.clone()));
    }

    fn add_proceed_stmt(method: &mut ImplItemMethod, a: &mut MethodAdviceAspectCodegen) {
        let stmt = Self::create_proceed_stmt_with_args(method, a);

        if stmt.is_err() {
            info!("Could not add proceed statement...");
        }
        stmt.ok().map(|p| {
            info!("Adding proceed statement created: {}.", SynHelper::get_str(&p));
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
        let proceed = quote! {
            let found = self.#ident(#(#args),*);
        };

        info!("Adding proceed statement: {}.", SynHelper::get_str(&proceed));

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
            info!("Adding statements {} to method.", SynHelper::get_str(before.clone()));
            let mut before_stmts = before.stmts;
            for i in 0..before_stmts.len() {
                info!("Adding statement {} to method.", SynHelper::get_str(before_stmts.get(i).unwrap().clone()));
                method.block.stmts.insert(i, before_stmts.get(i).unwrap().to_owned())
            }
            info!("Here are statements after: {}", SynHelper::get_str(method.block.clone()));
        });
    }

}
