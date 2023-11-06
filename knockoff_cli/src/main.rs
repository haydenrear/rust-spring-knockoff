use std::{env, fs};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use cargo::core::compiler::CompileMode;
use cargo::core::{Package, PackageId, PackageSet, Source, SourceId, SourceMap, Workspace};
use cargo::{Config, ops};
use cargo::ops::CompileOptions;
use cargo::sources::RegistrySource;
use cargo::util::{Filesystem, IntoUrl};
use toml::{Table, Value};
use toml::macros::{Deserialize, IntoDeserializer};
use codegen_utils::{get_build_project_dir, get_project_base_dir};
use crate::args_parser::ArgsParser;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;

import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/knockoff_cli.log"));

pub mod args_parser;

pub static MODULE_MACRO_CODEGEN_DEV: &'static str = "module_macro_codegen = { version = \"{}\", registry = \"estuary\", path = \"../../module_macro_codegen\" }";
pub static MODULE_PRECOMPILE_CODEGEN_DEV: &'static str = "module_precompile_codegen = { version = \"{}\", registry = \"estuary\", path = \"../../module_precompile_codegen\" }";
pub static MODULE_MACRO_CODEGEN: &'static str = "module_macro_codegen = { version = \"{}\", registry = \"estuary\" }";
pub static MODULE_PRECOMPILE_CODEGEN: &'static str = "module_precompile_codegen = { version = \"{}\", registry = \"estuary\" }";


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
    let _ = fs::create_dir_all(get_target_directory()).map_err(|e| {
        println!("Found error when creating directories in knockoff cli: {:?}", e);
    });
    compile_module_macro_codegen_gen_codegen(&args);
    if mode_arg.is_some() && mode_arg.unwrap() == "knockoff_dev" {
        compile_module_macro_lib_knockoff_dev(&args);
    } else {
        download_packages(&args);
        update_toml_values();
    }
}

fn download_packages(args: &HashMap<String, String>) {
    let packages = packages();
    packages.iter().for_each(|package_name| {
        let config = Config::default().unwrap();
        download_cargo_crate_to_directory(
            &get_registry_source_id(args),
            package_name,
            config,
            args
        );
    });
}

fn packages() -> [&'static str; 32] {
    let packages = [
        "aspect_knockoff_provider",
        "authentication_gen",
        "build_lib",
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
        "authentication_codegen"
    ];
    packages
}

// generate knockoff_builder crate and compile it, which will trigger generation of knockoff_codegen
// crate itself.
fn compile_module_macro_codegen_gen_codegen(args: &HashMap<String, String>) {
    generate_knockoff_builder(args);
    compile_in_target("knockoff_builder", args);
}

fn update_toml_values() {
    let packages_ref = packages()
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    for package in packages().iter() {
        if update_toml_value(package, &packages_ref) { continue; }
    }
    let items = fs::read_dir(&get_target_directory());
    if items.is_ok() {
        let packages_ref = items.unwrap().into_iter()
            .flat_map(|i| i.ok().into_iter())
            .flat_map(|i| i.file_name().to_str().map(|s| s.to_string()).into_iter())
            .filter(|i| !packages().contains(&i.as_str()) && get_target_directory().join(i).join("Cargo.toml").exists())
            .collect::<Vec<String>>();
        info!("Updating deps for {:?}", packages_ref);
        for p in packages().iter() {
            if update_toml_value(*p, &packages_ref) {
                continue;
            }
        }

    }
}

fn update_toml_value(package: &str, packages: &Vec<String>) -> bool {
    let toml_package = get_package_toml(package);
    if toml_package.is_some() {
        let mut toml_table = toml_package.unwrap();
        info!("Updating path for {}.", package);
        let dep = update_path_for(package, &mut toml_table, "dependencies", packages);
        let build_deps = update_path_for(package, &mut toml_table, "build-dependencies", packages);
        toml_table.get_mut("package").as_mut().map(|pkg| {
            pkg.as_table_mut().map(|package_table| {
                package_table.remove("version");
                package_table.insert("version".to_string(), Value::String("0.1.6".to_string()));
            })
        });
        if dep && build_deps {
            return true;
        }
        info!("Writing toml string for {:?}", &toml_table);
        let _ = toml::to_string(&toml_table)
            .map(|manifest_written| {
                let mut out_package = get_package_manifest_file(package);
                if out_package.as_ref().is_some() {
                    info!("Writing manifest {:?} to {:?}", manifest_written,
                            out_package.as_ref().unwrap());
                    let _ = out_package.as_mut()
                        .unwrap()
                        .write_all(manifest_written.as_bytes())
                        .map_err(|e| {
                            error!("Error writing manifest {:?}", e);
                        });
                    if !manifest_written.contains("[workspace]") {
                        let _ = out_package.as_mut().unwrap()
                            .write("[workspace]".to_string().as_bytes())
                            .map_err(|e| {
                                error!("Error writing manifest!");
                            });
                    }
                }
            })
            .map_err(|e| {
                error!("Error writing manifest for {}: {:?}", package, e);
            });
    } else {
        error!("Could not open toml package for {}", package);
    }
    false
}

fn update_path_for(
    package: &str,
    toml_table: &mut Table,
    path_type: &str,
    packages: &Vec<String>
) -> bool {
    let mut deps = toml_table.get_mut(path_type);
    if deps.as_ref().is_none() {
        return true
    }
    let mut deps = deps.unwrap();
    for other_package in packages.iter() {
        if package == *other_package {
            continue
        }
        deps.as_table_mut().map(|dependencies_table| {
            if dependencies_table.contains_key(other_package) {
                dependencies_table.get_mut(other_package).as_mut()
                    .map(|package_found| {
                        package_found.as_table_mut().map(|dep_package_table| {
                            info!("Inserting path for {}", package);
                            dep_package_table.insert("path".to_string(), Value::String(format!("../{}", other_package)));
                            dep_package_table.remove("registry-index");
                            dep_package_table.remove("version");
                        })
                    });
            }
        });
    }
    false
}

fn get_package_manifest_file(package_name: &str) -> Option<File> {
    let package = get_target_directory()
        .join(package_name)
        .join("Cargo.toml");
    if package.exists() {
        File::create(package)
            .map_err(|e| {
                error!("{} did not exist when attempted to open manifest.", package_name);
            })
            .ok()
    } else {
        None
    }
}

fn get_package_toml(package_name: &str) -> Option<Table> {
    let package = get_target_directory()
        .join(package_name)
        .join("Cargo.toml");
    if package.exists() {
        let mut cargo_manifest = File::open(&package);
        if !cargo_manifest.is_err() {
            let mut cargo_manifest_file = cargo_manifest.unwrap();
            let parsed_value = codegen_utils::parse::read_file_to_str(&mut cargo_manifest_file)
                .map_err(|e| {
                    error!("Error reading file {}: {:?}", package_name, e);
                });
            if parsed_value.is_err() {
                None
            } else {
                let out = toml::from_str::<Table>(parsed_value.unwrap().as_str()).ok();
                out
            }
        } else {
            error!("Error opening file {}", package_name);
            None
        }
    } else {
        None
    }
}

fn compile_packages(args: &HashMap<String, String>) {
    compile_in_target("module_macro", args);
    compile_in_target("module_precompile", args);
}

fn compile_module_macro_lib_knockoff_dev(args: &HashMap<String, String>) {
    let module_macro_lib = get_target_directory()
        .join("module_macro_lib")
        .join("Cargo.toml");
    compile_from_proj_directory(&module_macro_lib, args);
}

fn get_registry_source_id(args: &HashMap<String, String>) -> SourceId {
    args.get("registry-uri")
        .or(env::var("MODULE_MACRO_REGISTRY_INDEX_URI").ok().as_ref())
        .map(|registry_uri| SourceId::for_registry(
            &registry_uri.into_url().unwrap()
            ).ok()
        )
        .flatten()
        .or(Some(SourceId::crates_io(&Config::default().unwrap()).unwrap()))
        .unwrap()
}

fn update_cargo_path(cargo_path: &Path, dep_name: &str, path: &str) {
    let mut cargo_str = read_current_cargo_file(&cargo_path);
    let cargo_table = update_dependency_table(dep_name, path, &mut cargo_str);
    rewrite_cargo_file(cargo_path, cargo_table);
}

fn read_current_cargo_file(cargo_path: &&Path) -> String {
    let mut cargo_toml = File::open(cargo_path)
        .expect(
            &format!("Cargo file did not exist when trying to update {}.",
                     cargo_path.to_str().unwrap())
        );
    let mut cargo_str = "".to_string();
    cargo_toml.read_to_string(&mut cargo_str)
        .expect(
            &format!("Could not read cargo file for path: {}",
                     cargo_path.to_str().unwrap())
        );
    cargo_str
}

fn rewrite_cargo_file(cargo_path: &Path, cargo_table: Table) {
    let output = cargo_table.to_string();
    fs::remove_file(cargo_path).unwrap();
    File::create(cargo_path)
        .as_mut()
        .map(|file| file.write_all(output.as_bytes())
            .expect("Could not write toml.")
        )
        .expect("Could not write toml");
}

fn update_dependency_table(dep_name: &str, path: &str, cargo_str: &mut String) -> Table {
    let mut cargo_table = cargo_str.parse::<Table>().unwrap();
    cargo_table.insert("workspace".to_string(), Value::Table(Table::default()));
    cargo_table["dependencies"]
        .as_table_mut()
        .map(|t| {
            if t.contains_key(dep_name) {
                t[dep_name]
                    .as_table_mut()
                    .map(|knockoff_dep| {
                        knockoff_dep.insert("path".to_string(), Value::String(path.to_string()));
                        knockoff_dep.remove("version");
                    });
            }
            update_module_macro_data(t, "module_macro");
            update_module_macro_data(t, "module_macro_lib");
            update_module_macro_data(t, "module_precompile");
            update_module_macro_data(t, "knockoff_precompile_gen");
        });
    cargo_table
}

fn update_module_macro_data(t: &mut Table, macro_type: &str) {
    if t.contains_key(macro_type) {
        t[macro_type]
            .as_table_mut()
            .map(|module| {
                module.remove("registry-index");
                module.remove("version");
                module.insert("path".to_string(), Value::String(format!("../{}", macro_type).to_string()))
            });
    }
}

fn download_cargo_crate_to_directory(
    registry_id: &SourceId,
    module_macro_lib: &str,
    config: Config,
    args: &HashMap<String, String>
) {
    let version = get_version(args);
    download_cargo_crate(module_macro_lib, &version, registry_id, config)
        .map(|pkg|
            copy_cargo_crate(
                get_target_directory().join(module_macro_lib),
                pkg
            ));
}

fn get_project_path(path: &str) -> PathBuf {
    codegen_utils::env::get_project_path(path)
}

fn get_target_directory() -> PathBuf {
    get_project_path("target")
}

fn copy_cargo_crate(target_pkg_path: PathBuf, registry_pkg: Package) {
    let registry_pkg_path = registry_pkg.root();
    fs::create_dir_all(&target_pkg_path).unwrap();
    copy_dir(&registry_pkg_path, &target_pkg_path).unwrap();
}

fn download_cargo_crate<'a>(name: &str, version: &str, source_id: &SourceId, config: Config) -> Option<Package> {

    let whitelist = HashSet::new();
    let mut source = RegistrySource::remote(source_id.clone(), &whitelist, &config).unwrap();
    let mut source_map = SourceMap::new();

    source_map.insert(Box::new(source));

    let package_id = PackageId::new(name, version, source_id.clone()).unwrap();

    let p = PackageSet::new(
        &[package_id.clone()], source_map, &config
    ).unwrap();

    let mut downloaded = p.enable_download().unwrap();
    downloaded.start(package_id)
        .expect("Could not start downloading package");
    while downloaded.remaining() != 0 {
        downloaded.wait()
            .expect("Could not wait for package");
    }
    let package = p.get_one(package_id).ok().cloned();
    package
}

fn copy_dir(src_path: &Path, dst_path: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src_path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name();
        let dst_entry_path = dst_path.join(entry_name);
        if entry_path.is_file() {
            fs::copy(entry_path, dst_entry_path)?;
        } else {
            fs::create_dir_all(&dst_entry_path)?;
            copy_dir(&entry_path, &dst_entry_path)?;
        }
    }
    Ok(())
}

fn compile_in_target(dep_name_in_target: &str, args: &HashMap<String, String>) {
    let manifest_path = get_target_directory()
        .join(dep_name_in_target)
        .join("Cargo.toml");
    compile_from_proj_directory(&manifest_path, args);
}

fn compile_from_proj_directory(manifest_path: &Path, args: &HashMap<String, String>) {
    let config = Config::default().unwrap();
    let mut workspace = Workspace::new(&manifest_path, &config).unwrap();
    let file_system =  Filesystem::new(get_target_directory());
    workspace.set_target_dir(file_system);
    let mut compile_opts = CompileOptions::new(&workspace.config(), CompileMode::Build).unwrap();
    compile_opts.target_rustc_args.as_mut()
        .map(|mut compile_args|
            compile_args.push("rustc-env=PROJECT_BASE_DIRECTORY=".to_string() + get_project_base_dir().as_str())
        );
    let compile_result = ops::compile(&workspace, &compile_opts);
    if compile_result.is_err() {
        println!("Compilation completed with warnings.");
    } else {
        println!("Compilation completed successfully.");
    }
}

fn generate_knockoff_builder(args: &HashMap<String, String>) {
    // Create the directory structure for the new crate
    let version = &get_version(args);
    let knockoff_builder_der = get_target_directory().join("knockoff_builder");
    let src_dir = knockoff_builder_der.join("src");
    let lib_file = src_dir.join("lib.rs");
    let module_macro_codegen_dependency = args.get("mode")
        .filter(|mode| mode.as_str() == "knockoff_dev")
        .map(|_| format!("{}\n{}\n",
                         MODULE_MACRO_CODEGEN_DEV.replace("{}", version),
                         MODULE_PRECOMPILE_CODEGEN_DEV.replace("{}", version))
        )
        .or(Some(format!("{}\n{}\n",
                         MODULE_MACRO_CODEGEN.replace("{}", version),
                         MODULE_PRECOMPILE_CODEGEN.replace("{}", version)))
        )
        .unwrap();

    fs::create_dir_all(&src_dir)
        .unwrap();

    // Generate the main.rs file with the dependency on module_macro_codegen
    // Generate the lib.rs file with an empty module
    File::create(lib_file)
        .unwrap();

    // Create the Cargo.toml file
    let mut cargo_toml = File::create(knockoff_builder_der.join("Cargo.toml")).unwrap();
    cargo_toml.write_all(
        format!(
            "[package]\nname = \"knockoff_builder\"\nversion = \"{}\"\n\n[dependencies]\n{}\n[workspace]",
            version, module_macro_codegen_dependency
        ).as_bytes()
    ).expect("Could not write codegen.");

}

fn get_version(args: &HashMap<String, String>) -> String {
    let fallback_version = "0.1.5".to_string();
    let version = args.get(&"version".to_string())
        .or(Some(&fallback_version)).unwrap();
    version.clone()
}

#[test]
pub fn try_compile_cargo() {
    compile_in_target("knockoff_builder", &HashMap::new());
}

#[test]
pub fn test_generate_knockoff_builder() {
    generate_knockoff_builder(&HashMap::new());
    assert!(Path::new(concat!(project_directory!(), "target/knockoff_builder/src/lib.rs")).exists());
}

#[test]
fn test_download_lib() {
    let config = Config::default().unwrap();
    let source_id = SourceId::crates_io(&config).unwrap();
    let result = download_cargo_crate("a10", "0.0.0", &source_id, config);
    assert!(result.is_some())
}


#[test]
fn copy_dir_test() {
    let source_dir = get_build_project_dir("target/knockoff_providers_gen");
    let knockoff_providers_gen = Path::new(source_dir.as_str());
    let target = get_build_project_dir("target/test");
    let target = Path::new(target.as_str());
    fs::create_dir_all(target).unwrap();
    copy_dir(knockoff_providers_gen, target).unwrap();
    assert!(target.join("Cargo.toml").exists());
    fs::remove_dir_all(target).unwrap();
}

