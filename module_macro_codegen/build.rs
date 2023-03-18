use std::{env, fs};
use std::fmt::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use codegen_utils::{parse, project_directory, syn_helper};
use codegen_utils::env::get_project_base_dir;
use knockoff_logging::{create_logger_expr, initialize_log, initialize_logger, use_logging};
use std::io::Write;
use factories_codegen::factories_parser::{Dependency, FactoriesParser};
use factories_codegen::parse_provider::ParseProvider;
use factories_codegen::provider::{Provider, ProviderItem};
use factories_codegen::provider::DelegatingProvider;

use_logging!();
initialize_logger!(TextFileLoggerImpl, StandardLogData, concat!(project_directory!(), "log_out/module_macro_codegen_build_rs.log"));
initialize_log!();

/// The token stream providers need to depend on user provided crate, so that means we need to
/// generate a crate that depends on those user provided crates. We will then delegate to the user
/// provided dependency in that generated crate, which imports into the module macro codegen lib
/// to generate tokens dynamically from user, with the ProfileTree as a dependency provided to the user
/// or other library author to generate the tokens from.
fn main() {
     let parsed_factories = <FactoriesParser as DelegatingProvider>::deps();
     let knockoff_providers_dep = parsed_factories
         .iter().map(|provider| provider.dep_name.as_str())
         .collect::<Vec<&str>>();

     let mut directory_tuple = get_directories();
     fs::create_dir_all(&directory_tuple.0).unwrap();

     create_cargo_toml(&directory_tuple.1, knockoff_providers_dep, &parsed_factories);
     create_lib_rs(directory_tuple);

     let mut proj_dir = get_project_base_dir();
     let mut cargo_change = "cargo:rerun-if-changed=".to_string();
     cargo_change += proj_dir.as_str();
     println!("{}", cargo_change);
}

fn create_lib_rs(mut directory_tuple: (String, String, String)) {
     let lib_rs_file_path = Path::new(directory_tuple.2.as_str());

     fs::remove_file(lib_rs_file_path);

     File::create(lib_rs_file_path).ok()
         .map(|mut lib_rs_file| write_lib_rs(&mut lib_rs_file))
         .flatten().or_else(|| {
               log_message!("Could not write to lib.rs file.");
               None
          });
}

fn write_lib_rs(mut lib_rs_file: &mut File) -> Option<()> {
     let parsed_factories = <FactoriesParser as DelegatingProvider>::tokens();
     writeln!(&mut lib_rs_file, "{}", parsed_factories.to_string().as_str())
         .ok()
}

fn get_directories() -> (String, String, String) {
     let out_directory = env::var("CARGO_TARGET_DIR").ok().map(|r| {
          log_message!("{} is the target dir.", &r);
          r
     }).or_else(|| {
          log_message!("Could not find env with name CARGO_TARGET_DIR.");
          let mut project_dir = get_project_base_dir();
          project_dir += "target";
          Some(project_dir)
     }).map(|project_dir| {
          log_message!("{} is project directory", project_dir);
          project_dir
     }).unwrap();

     log_message!("{} is out", &out_directory);
     let mut out_lib_dir = out_directory.clone();
     out_lib_dir += "/knockoff_providers_gen/src";
     let mut cargo_toml = out_directory.clone();
     cargo_toml += "/knockoff_providers_gen/Cargo.toml";
     let mut out_lib_rs = out_directory.clone();
     out_lib_rs += "/knockoff_providers_gen/src/lib.rs";

     (out_lib_dir, cargo_toml, out_lib_rs)
}

fn create_cargo_toml(cargo_file: &str, knockoff_providers_dep: Vec<&str>, parsed_factories: &Vec<ProviderItem>) {
     let path = Path::new(cargo_file);
     if path.exists() {
          fs::remove_file(path).unwrap();
     }
     log_message!("Opening {}", &cargo_file);
     let mut cargo_file = File::create(path).unwrap();
     writeln!(&mut cargo_file, "{}", get_starting_toml_prelude().as_str()).unwrap();
     writeln!(&mut cargo_file, "[dependencies]").unwrap();
     let factories_dep = FactoriesParser::write_cargo_dependencies(&parsed_factories);
     writeln!(&mut cargo_file, "{}", factories_dep).unwrap();
     knockoff_providers_dep.iter().for_each(|p| {
          writeln!(&mut cargo_file, "[dependencies.{}]", p)
              .expect("Could not write to cargo file for knockoff_codegen");
          writeln!(&mut cargo_file, "path = \"../../{}\"", p)
              .expect("knockoff_codegen");
     });
     writeln!(&mut cargo_file, "[dependencies.module_macro_shared]")
         .unwrap();
     writeln!(&mut cargo_file, "path = \"../../module_macro_shared\"")
         .unwrap();
}

fn get_starting_toml_prelude() -> String {
     let mut prelude =

"[package]
name = \"knockoff_providers_gen\"
version = \"0.1.0\"
edition = \"2021\"
";

     prelude.to_string()
}
