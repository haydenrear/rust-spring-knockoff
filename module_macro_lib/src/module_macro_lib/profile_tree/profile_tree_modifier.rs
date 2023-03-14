use std::collections::HashMap;
use quote::{quote, ToTokens};
use syn::{parse2, Type};
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::Bean;
use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use module_macro_shared::profile_tree::ProfileTree;

pub trait ProfileTreeModifier {
    fn modify_bean(&self, dep_type: &mut Bean, profile_tree: &mut ProfileTree);
    fn new(profile_tree_items: &HashMap<String,Bean>) -> Self
    where Self: Sized;
}