use std::env;
use factories_codegen::factories_parser::{FactoriesParser, Phase};
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use codegen_utils::{get_build_project_dir, get_project_base_build_dir, get_project_dir};

import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/precompile_codegen.log"));

/// Iterates over the program to produce mutables, then generates the crate with those mutables.
fn main() {
     /// generate the mutables here, to be imported into precompile
     let knockoff_version = env::var("KNOCKOFF_VERSIONS")
         .or::<String>(Ok("0.1.5".into())).unwrap();
     let knockoff_factories = env::var("KNOCKOFF_FACTORIES")
         .ok()
         .or(Some(get_project_dir("codegen_resources/knockoff_factories.toml")))
         .unwrap();

     let out_directory = get_build_project_dir("target");

     FactoriesParser::write_phase(&knockoff_version, &knockoff_factories,
                                  &out_directory, &Phase::DFactory);
     let mut proj_dir = get_project_base_build_dir();
     let mut cargo_change = "cargo:rerun-if-changed=".to_string();
     cargo_change += proj_dir.as_str();
     println!("{}", cargo_change);
}

