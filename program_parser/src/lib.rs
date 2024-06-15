use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;

import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/program_parser.log"));

pub mod module_locator;
pub mod module_iterator;