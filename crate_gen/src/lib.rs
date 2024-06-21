use std::{env, fs};
use std::fs::File;
use std::path::Path;
use std::io::Write;
use codegen_utils::get_project_base_build_dir;
use codegen_utils::project_directory;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;


import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/codegen_utils.log"));

pub mod lib_writer;
pub use lib_writer::*;
pub mod toml_writer;
pub use toml_writer::*;

pub mod crate_writer;
pub use crate_writer::*;

pub use cargo_utils::*;


#[cfg(test)]
mod test;

