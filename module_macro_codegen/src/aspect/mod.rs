use std::ops::Deref;
use std::process::id;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, FnArg, Item, ItemFn, Type};
use knockoff_logging::{use_logging};
use crate::parser::CodegenItem;

use_logging!();

use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

#[derive(Clone, Default)]
pub struct MethodAdviceAspect {
    pub default: Option<TokenStream>,
    pub item: Option<Item>,
    pub type_found: Option<Type>,
    pub before_advice: TokenStream,
    pub after_advice: TokenStream,
    pub aspect_matcher_string: Vec<String>
}

impl MethodAdviceAspect {
    pub(crate) fn create_aspect_matcher(&self) -> AspectMatcher {
        Self::create_aspect(&self.aspect_matcher_string, Ident::new("TestAspect", Span::call_site()))
    }

    pub(crate) fn create_aspect(path: &Vec<String>, ident: Ident) -> AspectMatcher {
        AspectMatcher::new(path, ident)
    }
    pub(crate) fn new(item: &Item) -> Option<Box<dyn CodegenItem>> {
        Some(
            Box::new(
                match item {
                    Item::Fn(item_fn) => {
                        let ty = item_fn.sig.inputs.iter().flat_map(|i| {
                            match i {
                                FnArg::Receiver(_) => {
                                    vec![None]
                                }
                                FnArg::Typed(typed) => {
                                    if typed.ty.to_token_stream().to_string().as_str().contains("One") {
                                        return vec![Some(typed.ty.deref().clone())];
                                    }
                                    vec![]
                                }
                            }
                        }).flatten().next();
                        MethodAdviceAspect {
                            default: None,
                            item: Some(item.clone()),
                            type_found: ty,
                            before_advice: Default::default(),
                            after_advice: Default::default(),
                            aspect_matcher_string: vec![],
                        }
                    }
                    _ => {
                        MethodAdviceAspect::default()
                    }
                }
            )
        )
    }
}

/// Aspect matcher matches all structs/impls in particular modules and packages, and allows for
/// matching based on the struct name.
pub(crate) struct AspectMatcher {
    module_path: Vec<String>,
    struct_path: Ident
}

impl AspectMatcher {
    fn new(module_path: &Vec<String>, struct_path: Ident) -> Self {
        Self {
            module_path: module_path.clone(), struct_path
        }
    }
}

impl CodegenItem for MethodAdviceAspect {

    fn supports_item(item: &Item) -> bool where Self: Sized {
        match item {
            Item::Fn(item_fn) => {
                Self::is_aspect(&item_fn.attrs)
            }
            Item::Mod(mod_aspect) => {
                Self::is_aspect(&mod_aspect.attrs)
            }
            _ => {
                false
            }
        }
    }

    fn supports(&self, item: &Item) -> bool {
        match item {
            Item::Fn(item_fn) => {
                MethodAdviceAspect::is_aspect(&item_fn.attrs)
            }
            _ => {
                false
            }
        }
    }

    fn get_codegen(&self) -> Option<String> {
        None
    }

    fn default_codegen(&self) -> String {
        String::default()
    }

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        Box::new(self.clone())
    }

    fn get_unique_id(&self) -> String {
        "MethodAdviceAspect".to_string()
    }
}

impl MethodAdviceAspect {
    fn is_aspect(vec: &Vec<Attribute>) -> bool {
        vec.iter()
            .any(|attr|
                attr.to_token_stream().to_string().as_str().contains("aspect")
            )
    }
}

