// #![feature(file_create_new)]

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;

import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/cargo_utils.log"));

pub mod compile_cargo;
pub use compile_cargo::*;

pub mod download_cargo;
pub use download_cargo::*;

pub mod cargo_toml_utils;
pub use cargo_toml_utils::*;


#[cfg(test)]
pub mod test;