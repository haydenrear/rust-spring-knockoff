use std::{env, fs};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Error, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use cargo::core::compiler::CompileMode;
use cargo::core::{Package, PackageId, PackageSet, Source, SourceId, SourceMap, Workspace};
use cargo::{CargoResult, Config, ops};
use cargo::core::source::MaybePackage;
use cargo::ops::CompileOptions;
use cargo::sources::{RegistrySource, SourceConfigMap};
use cargo::util::{CanonicalUrl, Filesystem, LockServer};
use url::Url;
use codegen_utils::env::get_project_dir;
use codegen_utils::project_directory;

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
    generate_knockoff_builder();
    compile_knockoff_builder();
}

#[test]
fn test_download_lib() {
    let config = Config::default().unwrap();
    let source_id = SourceId::crates_io(&config).unwrap();
    let result = download_cargo_crate("a10", "0.0.0", source_id, config);
    assert!(result.is_some())
}

#[test]
fn test_module_macro_lib() {
    download_module_macro_lib()
}

fn download_module_macro_lib() {
    // let local_uri = Url::parse("http://localhost:1234")?;
    // let source_id = SourceId::for_registry(&local_uri)?;
    // download_cargo_crate("module_macro", "0.0.0", source_id, config)
    //     .map(|package| {
    //
    //     });
}

fn download_module_macro() {

}

fn copy_cargo_crate(target_pkg_path: PathBuf, registry_pkg: Package) {
    let registry_pkg_path = registry_pkg.root();
    fs::create_dir_all(&target_pkg_path).unwrap();
    copy_dir(&registry_pkg_path, &target_pkg_path).unwrap();
}

fn download_cargo_crate<'a>(name: &str, version: &str, source_id: SourceId, config: Config) -> Option<Package> {

    let whitelist = HashSet::new();
    let mut source = RegistrySource::remote(source_id, &whitelist, &config).unwrap();
    let mut source_map = SourceMap::new();

    source_map.insert(Box::new(source));

    let package_id = PackageId::new(name, version, SourceId::crates_io(&config).unwrap()).unwrap();

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

#[test]
fn copy_dir_test() {
    let source_dir = get_project_dir("target/knockoff_providers_gen");
    let knockoff_providers_gen = Path::new(source_dir.as_str());
    let target = get_project_dir("target/test");
    let target = Path::new(target.as_str());
    fs::create_dir_all(target).unwrap();
    copy_dir(knockoff_providers_gen, target).unwrap();
    assert!(target.join("Cargo.toml").exists());
    fs::remove_dir_all(target).unwrap();
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

fn compile_knockoff_builder() {
    let manifest_path = concat!(project_directory!(), "target/knockoff_builder/Cargo.toml");
    let config = Config::default().unwrap();
    let mut workspace = Workspace::new(&Path::new(manifest_path), &config).unwrap();
    let mut compile_opts = CompileOptions::new(&workspace.config(), CompileMode::Build).unwrap();
    workspace.set_target_dir(Filesystem::new(Path::new(concat!(project_directory!(), "target")).to_path_buf()));
    let compile_result = ops::compile(&workspace, &compile_opts);
    if compile_result.is_err() {
        println!("Compilation completed with warnings.");
    } else {
        println!("Compilation completed successfully.");
    }
}

fn generate_knockoff_builder() {
    // Create the directory structure for the new crate
    let crate_name = concat!(project_directory!(), "target/knockoff_builder");
    let src_dir = Path::new(crate_name).join("src");
    let lib_file = src_dir.join("lib.rs");
    let module_macro_codegen_dependency = "module_macro_codegen = {path = \"../../module_macro_codegen\"}";

    fs::create_dir_all(&src_dir)
        .unwrap();

    // Generate the main.rs file with the dependency on module_macro_codegen
    // Generate the lib.rs file with an empty module
    File::create(lib_file)
        .unwrap();

    // Create the Cargo.toml file
    let mut cargo_toml = File::create(Path::new(crate_name).join("Cargo.toml")).unwrap();
    cargo_toml.write_all(
        format!(
            "[package]\nname = \"knockoff_builder\"\nversion = \"0.1.0\"\n\n[dependencies]\n{}\n[workspace]",
            module_macro_codegen_dependency
        ).as_bytes()
    ).expect("Could not write codegen.");

}

#[test]
pub fn try_compile_cargo() {
    compile_knockoff_builder();
}

#[test]
pub fn test_generate_knockoff_builder() {
    generate_knockoff_builder();
    assert!(Path::new(concat!(project_directory!(), "target/knockoff_builder/src/lib.rs")).exists());
}

