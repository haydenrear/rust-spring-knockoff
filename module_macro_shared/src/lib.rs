#![feature(let_chains)]

pub mod util;
pub use util::*;
pub mod item_parser;
pub use item_parser::*;
pub mod module_parser;
pub use module_parser::*;
pub mod module_macro_shared_codegen;
pub use module_macro_shared_codegen::*;
pub mod profile_tree;
pub use profile_tree::*;
pub mod debug;
pub use debug::*;

pub mod bean_parser;
pub use bean_parser::*;
pub mod bean;
pub use bean::*;
pub mod dependency;
pub use dependency::*;
pub mod parse_container;
pub use parse_container::*;
pub mod functions;
pub use functions::*;
pub mod item_modifier;
pub use item_modifier::*;
pub mod module_tree;
pub use module_tree::*;
pub mod token_provider;
pub use token_provider::*;
pub mod metadata_parser;
pub use metadata_parser::*;
pub mod generics;
pub use generics::*;

pub mod profile_tree_builder;
pub use profile_tree_builder::*;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/module_macro_shared.log"));
