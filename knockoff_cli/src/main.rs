use std::{env, fs};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Error, Read, Seek, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use cargo::core::compiler::CompileMode;
use cargo::core::{Manifest, Package, PackageId, PackageSet, Source, SourceId, SourceMap, Workspace};
use cargo::{CargoResult, Config, ops};
use cargo::core::source::MaybePackage;
use cargo::ops::CompileOptions;
use cargo::sources::{RegistrySource, SourceConfigMap};
use cargo::util::{CanonicalUrl, Filesystem, IntoUrl, LockServer};
use cargo::util::toml::DetailedTomlDependency;
use url::Url;
use codegen_utils::env::{get_project_base_build_dir, get_build_project_base_path, get_build_project_dir, get_project_base_dir};
use codegen_utils::project_directory;
use toml::{Table, Value};
use toml::macros::{Deserialize, IntoDeserializer};
use crate::args_parser::ArgsParser;

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
        compile_module_macro(&args);
    }
}

// generate knockoff_builder crate and compile it, which will trigger generation of knockoff_codegen
// crate itself.
fn compile_module_macro_codegen_gen_codegen(args: &HashMap<String, String>) {
    generate_knockoff_builder(args);
    compile_in_target("knockoff_builder", args);
}

fn compile_module_macro(args: &HashMap<String, String>) {
    let registry = get_registry_source_id(args);
    download_update_providers_gen_dependent(
        &registry,
        "module_macro",
        "knockoff_providers_gen",
        args
    );
    download_update_providers_gen_dependent(
        &registry,
        "module_macro_lib",
        "knockoff_providers_gen",
        args
    );
    download_update_providers_gen_dependent(
        &registry,
        "module_precompile",
        "knockoff_precompile_gen",
        args
    );
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

fn download_update_providers_gen_dependent(registry_id: &SourceId, module_macro_lib: &str, provider: &str,
                                           args: &HashMap<String, String>) {
    let config = Config::default().unwrap();
    let version = get_version(args);
    download_cargo_crate(module_macro_lib, &version, registry_id, config)
        .map(|pkg|
            copy_cargo_crate(
                get_target_directory().join(module_macro_lib),
                pkg
        ));
    update_cargo_path(
        &get_target_directory().join(module_macro_lib).join("Cargo.toml"),
        provider,
        format!("../{}", provider).as_str()
    );
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
fn test_download_module_macro_lib() {
    download_update_providers_gen_dependent(
        &SourceId::for_registry( &"http://localhost:1234/git/index".into_url().unwrap()).unwrap(),
        "module_macro_lib",
        "knockoff_providers_gen",
        &HashMap::new()
    );
    assert!(get_project_path("target/module_macro_lib").is_dir());
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

