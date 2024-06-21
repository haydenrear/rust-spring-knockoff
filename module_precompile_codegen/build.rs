
use std::env;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use factories_codegen::factories_parser::{FactoriesParser, Phase};
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Item, ItemImpl, ItemMod, Visibility};
use toml::Table;
use codegen_utils::{FlatMapOptional, program_src, project_directory, project_directory_path};
use codegen_utils::{get_build_project_dir, get_project_base_build_dir, get_project_dir};
use codegen_utils::syn_helper::SynHelper;
use crate_gen::CrateWriter;
use dfactory_dcodegen::write_d_factory_crate;

import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/precompile_codegen.log"));

use knockoff_dfactory_gen::{DelegatingFrameworkTokenProvider, DelegatingParseContainerModifierProvider, DelegatingItemModifier,
                            DelegatingProfileTreeFinalizerProvider, DelegatingTokenProvider, DelegatingParseProvider};

use module_macro_shared::{BuildParseContainer, ModuleParser, parse_module_into_container, ParseContainer, ProfileProfileTreeModifier,
                          ProfileTreeBuilder, ProfileTreeModifier, ItemModifier, ProfileTreeFrameworkTokenProvider};
use optional::FlatMapResult;


fn main() {
    match write_d_factory_crate() {
        None => { info!("Failed to write dfactory crate."); },
        Some(_) => { info!("Wrote dfactory crate!"); }
    }

     let knockoff_version = env::var("KNOCKOFF_VERSIONS")
         .or::<String>(Ok("0.1.5".into())).unwrap();
     let knockoff_factories = env::var("KNOCKOFF_FACTORIES")
         .ok()
         .or(Some(get_project_dir("codegen_resources/knockoff_factories.toml")))
         .unwrap();

     let out_directory = get_build_project_dir("target");

     /// PreCompile imports from that which is generated in DFactory
     FactoriesParser::write_phase(&knockoff_version, &knockoff_factories,
                                  &out_directory, &Phase::PreCompile);

     let mut proj_dir = get_project_base_build_dir();
     let mut cargo_change = "cargo:rerun-if-changed=".to_string();
     cargo_change += proj_dir.as_str();
     println!("{}", cargo_change);
}

