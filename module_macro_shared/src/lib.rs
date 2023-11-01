#![feature(let_chains)]

pub mod module_macro_shared_codegen;
pub mod profile_tree;
pub mod debug;
pub mod bean;
pub mod dependency;
pub mod parse_container;
pub mod functions;
pub mod item_modifier;
pub mod module_tree;
pub mod token_provider;
pub mod metadata_parser;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/module_macro_shared.log"));
