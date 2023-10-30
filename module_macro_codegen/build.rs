use std::{env, fs};
use std::collections::HashMap;
use std::fmt::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use codegen_utils::env::{get_project_base_build_dir, get_build_project_dir};
use std::io::Write;
use toml::Table;
use factories_codegen::factories_parser::{Factories, FactoriesParser, Provider};
use factories_codegen::parse_provider::ParseProvider;
use factories_codegen::provider::DelegatingProvider;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/module_macro_codegen_build_rs.log"));

/// The token stream providers need to depend on user provided crate, so that means we need to
/// generate a crate that depends on those user provided crates. We will then delegate to the user
/// provided dependency in that generated crate, which imports into the module macro codegen lib
/// to generate tokens dynamically from user, with the ProfileTree as a dependency provided to the user
/// or other library author to generate the tokens from.
fn main() {
     let mut directory_tuple = get_create_directories();

     info!("Writing output directory: {:?}", &directory_tuple);

     FactoriesParser::write_cargo_toml(&directory_tuple.1, directory_tuple.3);
     FactoriesParser::write_tokens_lib_rs(directory_tuple.2);

     let mut proj_dir = get_project_base_build_dir();
     let mut cargo_change = "cargo:rerun-if-changed=".to_string();
     cargo_change += proj_dir.as_str();
     println!("{}", cargo_change);
}

fn get_create_directories() -> (String, String, String, String) {
     let base_dir = get_project_base_build_dir();
     let out_directory = get_build_project_dir("target");

     log_message!("{} is out", &out_directory);
     let mut out_lib_dir = out_directory.clone();
     out_lib_dir += "/knockoff_providers_gen/src";
     let mut cargo_toml = out_directory.clone();
     cargo_toml += "/knockoff_providers_gen/Cargo.toml";
     let mut out_lib_rs = out_directory.clone();
     out_lib_rs += "/knockoff_providers_gen/src/lib.rs";
     let _ = fs::remove_dir_all(&out_lib_dir)
         .map_err(|err| {
              write_error_creating_out_lib(&mut out_lib_rs, &err);
              Ok::<(), Error>(())
         });
     let _ = fs::create_dir_all(&out_lib_dir)
          .map_err(|err| {
               write_error_creating_out_lib(&mut out_lib_rs, &err);
               Ok::<(), Error>(())
          });

     (out_lib_dir, cargo_toml, out_lib_rs, base_dir)
}

fn write_error_creating_out_lib(mut out_lib_rs: &mut String, err: &dyn std::error::Error)  {
     error!("Tried to create {}. Error creating knockoff providers gen: {}.", out_lib_rs, err.to_string().as_str());
}


