use std::fs::File;
use std::path::PathBuf;
use std::string::ToString;
use codegen_utils::{project_directory, project_directory_path};

const out: &'static str = r#"
[package]
name = "cargo_utils"
version = "0.1.5"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
cargo = "0.69.0"
url = "2.3.1"
toml = "0.7.3"
lazy_static = "1.4.0"
[dependencies.codegen_utils]
version = "0.1.5"
registry = "estuary"
path = "../../codegen_utils"
[dependencies.knockoff_logging]
version = "0.1.5"
registry = "estuary"
path = "../../knockoff_logging"
[build-dependencies]
cargo = "0.69.0"
[build-dependencies.codegen_utils]
version = "0.1.5"
registry = "estuary"
path = "../../codegen_utils"
[build-dependencies.knockoff_logging]
version = "0.1.5"
registry = "estuary"
path = "../../knockoff_logging"
"#;

use std::io::Write;
use toml::Value;
use crate::cargo_toml_utils::update_toml::{TomlUpdates, UpdateToml};
use crate::{do_on_dir, get_key, SearchType};

#[test]
fn test_update_cargo_toml() {
    write_cargo();
    let mut updates_to = vec![];
    let packages = vec!["knockoff_logging".to_string(), "codegen_utils".to_string(), "cargo".to_string()];
    updates_to.push(TomlUpdates::RemoveKeyFromBuildDependencies("registry-index".to_string(), &packages));
    updates_to.push(TomlUpdates::RemoveKeyFromBuildDependencies("version".to_string(), &packages));
    updates_to.push(TomlUpdates::RemoveKeyFromDependencies("registry-index".to_string(), &packages));
    updates_to.push(TomlUpdates::RemoveKeyFromDependencies("version".to_string(), &packages));
    updates_to.push(TomlUpdates::ChangeKeyPackage("version".to_string(), "0.1.6".to_string()));
    packages.iter().for_each(|p| updates_to.push(TomlUpdates::AddPathToPackage(p.to_string(), p.to_string())));


    let buf = PathBuf::from(project_directory!()).join("cargo_utils").join("test_resources");
    let updates = UpdateToml::do_update_toml_from(&updates_to, "test_app", &buf, &packages);

    assert!(updates.toml_table.is_some());

    let table = updates.toml_table.unwrap();
    let codegen_utils = table.get("dependencies").unwrap().as_table().unwrap().get("codegen_utils");

    assert!(!codegen_utils.unwrap().as_table().unwrap().contains_key("version"));
    assert_eq!(codegen_utils.unwrap().as_table().unwrap().get("path").unwrap().as_str().unwrap(), "../codegen_utils");
    assert!(table.contains_key("package"));
}

fn write_cargo() {
    let path = PathBuf::from(project_directory!()).join("cargo_utils").join("test_resources").join("test_app").join("Cargo.toml");
    std::fs::remove_file(&path);
    println!("{:?}", path.to_str());
    let mut f = File::create_new(&path).or(File::open(&path)).unwrap();
    writeln!(&mut f, "{}", out).unwrap();
    drop(path);
    drop(f);
}

#[test]
fn test_do_on() {
    do_on_dir(PathBuf::from(project_directory!()).join("cargo_utils").join("test_resources"), &|pkgs| { println!("{:?}", pkgs); });
}

#[test]
fn do_test_get_key() {
    let proj = project_directory_path!().join("codegen_resources").join("default_phase_deps.toml");
    let found = codegen_utils::io_utils::read_dir_to_file(&proj).unwrap();
    let t: Value = toml::from_str(found.as_str()).unwrap();
    let found = get_key(vec![SearchType::FieldKey("all".to_string())], &t).unwrap();
    println!("{:?}", &found.to_string());
}