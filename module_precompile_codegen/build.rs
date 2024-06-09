use std::env;
use factories_codegen::factories_parser::{FactoriesParser, Phase};
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use codegen_utils::{get_build_project_dir, get_project_base_build_dir, get_project_dir};

import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/precompile_codegen.log"));

/// The token stream providers need to depend on user provided crate, so that means we need to
/// generate a crate that depends on those user provided crates. We will then delegate to the user
/// provided dependency in that generated crate, which imports into the module macro codegen lib
/// to generate tokens dynamically from user, with the ProfileTree as a dependency provided to the user
/// or other library author to generate the tokens from.

/// TODO: A new stage in the codegen is required to provide codegen inputs for framework dependencies, or rather
///     any arbitrary number of stages is required to be defined in knockoff_factories.toml with stage identifiers.
///     In this case, any number knockoff_provider_gens will be transitively depended on by knockoff_providers
///     and then pub use ** will be used in the original knockoff_providers_gen. So then this replaces
///     module_macro_codegen.
fn main() {
     let knockoff_version = env::var("KNOCKOFF_VERSIONS")
         .or::<String>(Ok("0.1.5".into())).unwrap();
     let knockoff_factories = env::var("KNOCKOFF_FACTORIES")
         .ok()
         .or(Some(get_project_dir("codegen_resources/knockoff_factories.toml")))
         .unwrap();

     let base_dir = get_project_base_build_dir();

     let out_directory = get_build_project_dir("target");

     FactoriesParser::write_phase(&knockoff_version, &knockoff_factories, &base_dir,
                                  &out_directory, &Phase::PreCompile)
         .map(|stages| {
              info!("Writing stages of factory: {:?}.", &stages);
              FactoriesParser::write_tokens_lib_rs(
                   stages, &out_directory, &knockoff_version, &Phase::PreCompile)
         });

     let mut proj_dir = get_project_base_build_dir();
     let mut cargo_change = "cargo:rerun-if-changed=".to_string();
     cargo_change += proj_dir.as_str();
     println!("{}", cargo_change);
}

