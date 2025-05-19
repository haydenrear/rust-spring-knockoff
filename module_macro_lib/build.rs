use std::env;
use std::io::Write;
use codegen_utils::project_directory;
use knockoff_logging::*;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::{get_build_project_dir, get_project_base_build_dir};
import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/module_macro_lib_build.log"));


fn main() {
    log_message!("Initializing module macro lib.");
    // let mut cargo_change = "cargo:rerun-if-changed=".to_string();
    // cargo_change += get_project_base_build_dir().as_str();
    // 
    // println!("{}", cargo_change);
}