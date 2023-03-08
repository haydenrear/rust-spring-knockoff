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
use crate::module_macro_lib::item_modifier::aspect_modifier::AspectModifier;
use crate::module_macro_lib::item_modifier::ItemModifier;

#[derive(Default)]
pub struct DelegatingItemModifier {
    modifiers: Vec<Box<dyn ItemModifier>>
}

impl DelegatingItemModifier {
    pub fn new() -> Self {
        Self {
            modifiers: vec![Box::new(AspectModifier{})]
        }
    }
}

impl ItemModifier for DelegatingItemModifier {

    fn modify_item(&self, parse_container: &mut ParseContainer, item: &mut Item, path_depth: Vec<String>) {
        let mut path_depth = path_depth.clone();
        self.modifiers.iter().for_each(|f| {
            if f.supports_item(&item) {
                f.modify_item(parse_container, item, path_depth.clone());
            }
        });
        match item {
            Item::Mod(ref mut item_mod) => {
                let mod_ident = item_mod.ident.to_string().clone();
                if !path_depth.contains(&mod_ident) {
                    path_depth.push(mod_ident);
                }
                item_mod.content.iter_mut().for_each(|c| {
                    for item in c.1.iter_mut() {
                        self.modify_item(parse_container, item, path_depth.clone())
                    }
                });
            }
            _ => {}
        }
    }

    fn supports_item(&self, item: &Item) -> bool {
        self.modifiers.iter().any(|f| f.supports_item(item))
    }
}
