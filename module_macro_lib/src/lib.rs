// include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

pub mod module_macro_lib {
    pub mod parse_container;
    pub mod knockoff_context_builder;
    pub mod profile_tree;
    pub mod parse_module;
    pub mod context_builder;
    pub mod knockoff_context;
    pub mod generics_provider;
    #[cfg(test)]
    pub mod test;
}

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/module_macro_lib.log"));



