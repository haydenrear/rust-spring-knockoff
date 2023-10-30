extern crate core;

use knockoff_security::knockoff_security::authentication_type::AuthenticationAware;

pub mod initializer;
pub mod parser;
pub mod field_aug;
pub mod authentication_type;
pub mod module_extractor;
pub mod codegen_item_macros;
pub mod test;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/module_macro_codegen.log"));
