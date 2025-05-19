use std::{env, fs};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;

use cargo::Config;
use cargo::core::{Package, Source, SourceId};
use cargo::util::IntoUrl;
use lazy_static::lazy_static;
use toml::{Table, Value};
use toml::macros::{Deserialize, IntoDeserializer};
use cargo_utils::cargo_toml_utils::update_toml::{TomlUpdates, UpdateToml};

use codegen_utils::{FlatMapOptional, get_project_base_dir, get_project_base_path};
use codegen_utils::project_directory;
use knockoff_logging::*;

use crate::args_parser::ArgsParser;

import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/knockoff_cli.log"));

pub mod args_parser;

#[cfg(test)]
pub mod test;

pub static MODULE_MACRO_CODEGEN_DEV: &'static str = "module_macro_codegen = { version = \"{}\", registry = \"estuary\", path = \"../../module_macro_codegen\" }";
pub static MODULE_PRECOMPILE_CODEGEN_DEV: &'static str = "module_precompile_codegen = { version = \"{}\", registry = \"estuary\", path = \"../../module_precompile_codegen\" }";
pub static MODULE_MACRO_CODEGEN: &'static str = "module_macro_codegen = { version = \"{}\", registry = \"estuary\" }";
pub static MODULE_PRECOMPILE_CODEGEN: &'static str = "module_precompile_codegen = { version = \"{}\", registry = \"estuary\" }";
pub static MODULE_DFACTORY_CODEGEN_DEV: &'static str = "dfactory_dcodegen_codegen = { version = \"{}\", registry = \"estuary\", path = \"../../dfactory_dcodegen_codegen\" }";
pub static MODULE_DFACTORY_CODEGEN: &'static str = "dfactory_dcodegen_codegen = { version = \"{}\", registry = \"estuary\" }";


/// Instruction:
/// 1. generate crate that uses module_macro_codegen and compile it to generate the knockoff_providers_gen
///    based on user provided factories.toml - both generated in target folder.
/// 2. copy module_macro_lib and module_macro to the target folder as well.
/// Description:
/// The module_macro_lib and module_macro are both pushed up to crates.io with toml pointing to generated
/// dependency crates located in the target directory. This allows for the service loader pattern in
/// codegen. It's possible to publish them without compiling using the following:
/// `cargo publish --no-verify`.
fn main() {
    let args = ArgsParser::parse_arg(env::args());
    let mode_arg = args.get("mode");
    let _ = fs::create_dir_all(get_target_directory())
        // .map_err(err::log_err("Found error when creating directories in knockoff cli: "));
        ;
    compile_module_macro_codegen_gen_codegen(&args);
    println!("Starting compile.");
    // if mode_arg.is_some() && mode_arg.unwrap() == "knockoff_dev" {
    //     println!("Compiling for knockoff dev.");
    //     compile_module_macro_lib_knockoff_dev(&args);
    //     compile_packages(&args);
    // } else {
    //     download_packages(&args);
    //     update_toml_values();
    // }
}

fn download_packages(args: &HashMap<String, String>) {
    let packages = packages();
    packages.iter().for_each(|package_name| {
        let config = Config::default().unwrap();
        cargo_utils::download_cargo_crate_to_directory(
            &cargo_utils::get_registry_source_id(args.get("registry-uri")),
            package_name,
            config,
            args,
            get_target_directory()
        );
    });
}

fn packages() -> Vec<&'static str> {
    let packages = vec![
        "aspect_knockoff_provider",
        "authentication_gen",
        "codegen_utils",
        "collection_util",
        "crate_gen",
        "data_framework",
        "factories_codegen",
        "handler_mapping",
        "knockoff_logging",
        "knockoff_security",
        "knockoff_helper",
        "knockoff_tokio_util",
        "module_macro_codegen",
        "module_macro_shared",
        "module_macro_lib",
        "mongo_repo",
        "security_parse_provider",
        "spring_knockoff_boot",
        "spring_knockoff_boot_macro",
        "wait_for",
        "web_framework",
        "web_framework_shared",
        "string_utils",
        "set_enum_fields",
        "knockoff_env",
        "configuration_properties_macro",
        "module_precompile",
        "module_precompile_codegen",
        "knockoff_resource",
        "module_macro",
        "authentication_codegen",
        "aspect_knockoff_gen"
    ];
    packages
}

// generate knockoff_builder crate and compile it, which will trigger generation of knockoff_codegen
// crate itself.
fn compile_module_macro_codegen_gen_codegen(args: &HashMap<String, String>) {
    generate_knockoff_builder(args);
    generate_precompile_builder(args);
    compile_in_sub_project("dfactory_dcodegen_codegen", args);
    compile_in_target("knockoff_builder", args);
    compile_in_target("knockoff_precompile_builder", args);
}

fn update_toml_values() {
    cargo_utils::cargo_toml_utils::do_on_dir(get_target_directory(), &|pkgs| update_packages_ref(pkgs))
}

fn update_packages_ref(packages_ref: &Vec<String>) {
    packages().iter()
        .filter(|f| !update_toml_value(**f, &packages_ref))
        .for_each(|f| { error!("Failed to update toml for {}", f); });
}

fn update_toml_value(package: &str, packages: &Vec<String>) -> bool {
    let mut updates_to: Vec<TomlUpdates> = vec![
        TomlUpdates::RemoveKeyFromBuildDependencies("registry-index".to_string(), packages),
        TomlUpdates::RemoveKeyFromBuildDependencies("version".to_string(), packages),
        TomlUpdates::RemoveKeyFromDependencies("registry-index".to_string(), packages),
        TomlUpdates::RemoveKeyFromDependencies("version".to_string(), packages),
        TomlUpdates::ChangeKeyPackage("version".to_string(), "0.1.6".to_string())
    ];

    packages.iter().for_each(|p| updates_to.push(TomlUpdates::AddPathToPackage(p.to_string(), p.to_string())));

    let tgt_dir = get_target_directory();
    let updates = UpdateToml::do_update_toml_from(&updates_to, package, &tgt_dir, packages);

    !(updates.toml_table.is_some()
        && updates.toml_table.map(|t| t.contains_key("build-dependencies") && t.contains_key("dependencies"))
                .or(Some(false)).unwrap())
}

fn compile_packages(args: &HashMap<String, String>) {
    compile_in_project("module_precompile", args);
    compile_in_project("module_macro", args);
}

fn compile_module_macro_lib_knockoff_dev(args: &HashMap<String, String>) {
    let module_macro_lib = get_project_base_path()
        .join("module_macro_lib")
        .join("Cargo.toml");
    compile_from_proj_directory(&module_macro_lib, args);
}

fn get_project_path(path: &str) -> PathBuf {
    codegen_utils::env::get_project_path(path)
}

fn get_target_directory() -> PathBuf {
    get_project_path("target")
}

fn download_cargo_crate<'a>(name: &str, version: &str, source_id: &SourceId, config: Config) -> Option<Package> {
    cargo_utils::download_cargo_crate(name, version, source_id, config)
}

fn compile_in_target(dep_name_in_target: &str, args: &HashMap<String, String>) {
    let manifest_path = get_target_directory()
        .join(dep_name_in_target)
        .join("Cargo.toml");
    compile_from_proj_directory(&manifest_path, args);
}

fn compile_in_sub_project(dep_name_in_target: &str, args: &HashMap<String, String>) {
    let manifest_path = get_project_base_path()
        .join(dep_name_in_target)
        .join("Cargo.toml");

    compile_from_sub_directory(&manifest_path, args);
}

fn compile_in_project(dep_name_in_target: &str, args: &HashMap<String, String>) {
    let manifest_path = get_project_base_path()
        .join(dep_name_in_target)
        .join("Cargo.toml");

    compile_from_proj_directory(&manifest_path, args);
    println!("Finished compiling {}", dep_name_in_target);
}

fn compile_from_proj_directory(manifest_path: &Path, args: &HashMap<String, String>) {
    cargo_utils::compile_from_directory(manifest_path, args, get_target_directory());
}

fn compile_from_sub_directory(manifest_path: &Path, args: &HashMap<String, String>) {
    cargo_utils::compile_from_directory(manifest_path, args, manifest_path.parent().unwrap().join("target").to_path_buf());
}

fn to_table(s: String) -> Result<Table, toml::de::Error> {
    Table::from_str(&s)
}

fn generate_precompile_builder(args: &HashMap<String, String>) {
    // Create the directory structure for the new crate
    let version = &cargo_utils::get_version(args);
    let module_macro_codegen_dependency = args.get("mode")
        .filter(|mode| mode.as_str() == "knockoff_dev")
        .flat_map_opt(|_| to_table(format!("{}\n{}\n",
                                           MODULE_DFACTORY_CODEGEN_DEV.replace("{}", version),
                                           MODULE_PRECOMPILE_CODEGEN_DEV.replace("{}", version)
        ))
            // .map_err(err::log_err("Failed to compile module macro codegen."))
            .ok()
        )
        .or(to_table(format!("{}\n{}\n",
                             MODULE_DFACTORY_CODEGEN.replace("{}", version),
                             MODULE_PRECOMPILE_CODEGEN_DEV.replace("{}", version)
        ))
            // .map_err(err::log_err("Failed to compile module macro codegen."))
            .ok()
        )
        .unwrap();

    crate_gen::CrateWriter::write_dependency_agg_crate("knockoff_precompile_builder", version, &get_target_directory(), &module_macro_codegen_dependency);

}



fn generate_knockoff_builder(args: &HashMap<String, String>) {
    // Create the directory structure for the new crate
    let version = &cargo_utils::get_version(args);
    let module_macro_codegen_dependency = args.get("mode")
        .filter(|mode| mode.as_str() == "knockoff_dev")
        .flat_map_opt(|_| to_table(format!("{}\n{}\n",
                                  MODULE_MACRO_CODEGEN_DEV.replace("{}", version),
                                  MODULE_DFACTORY_CODEGEN_DEV.replace("{}", version)))
            // .map_err(err::log_err("Failed to compile module macro codegen."))
            .ok()
        )
        .or(to_table(format!("{}\n{}\n",
                         MODULE_MACRO_CODEGEN.replace("{}", version),
                         MODULE_DFACTORY_CODEGEN.replace("{}", version)))
            // .map_err(err::log_err("Failed to compile module macro codegen."))
            .ok()
        )
        .unwrap();

    crate_gen::CrateWriter::write_dependency_agg_crate("knockoff_builder", version, &get_target_directory(), &module_macro_codegen_dependency);

}


