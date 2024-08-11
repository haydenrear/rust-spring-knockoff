use std::collections::HashMap;
use std::path::{Path, PathBuf};
use cargo::{Config, ops};
use cargo::core::compiler::CompileMode;
use cargo::core::Workspace;
use cargo::ops::CompileOptions;
use cargo::util::Filesystem;
use codegen_utils::get_project_base_dir;

use knockoff_logging::*;
use std::sync::Mutex;
use crate::logger_lazy;


import_logger!("compile_cargo.rs");

pub fn compile_from_directory(manifest_path: &Path, args: &HashMap<String, String>, target_directory: PathBuf) {
    let config = Config::default().unwrap();
    let mut workspace = Workspace::new(&manifest_path, &config).unwrap();
    let file_system =  Filesystem::new(target_directory);
    workspace.set_target_dir(file_system);
    let mut compile_opts = CompileOptions::new(&workspace.config(), CompileMode::Build).unwrap();
    compile_opts.target_rustc_args.as_mut()
        .map(|mut compile_args| {
            compile_args.push("rustc-env=PROJECT_BASE_DIRECTORY=".to_string() + get_project_base_dir().as_str());
            compile_args
        });
    let compile_result = ops::compile(&workspace, &compile_opts);
    if compile_result.is_err() {
        info!("Compilation completed with warnings: {:?}.", compile_result.err().unwrap());
    } else {
        info!("Compilation completed successfully.");
    }
}
