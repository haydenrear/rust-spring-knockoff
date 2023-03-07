use std::collections::HashMap;
use quote::ToTokens;
use knockoff_logging::{initialize_log, use_logging};
use crate::module_macro_lib::bean_parser::BeanDependencyParser;
use crate::module_macro_lib::profile_tree::ProfileTree;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::module_tree::{Bean, DepType, FunctionType};
use crate::module_macro_lib::parse_container::ParseContainer;

pub trait ProfileTreeOperation {
    fn do_operation(&self, parse_container: &mut ParseContainer);
}

