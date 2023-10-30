pub mod aspect_knockoff_provider;
mod test_aspect;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/factories_codegen.log"));
