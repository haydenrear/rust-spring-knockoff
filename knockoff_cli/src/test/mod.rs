use std::collections::HashMap;
use std::fs;
use std::path::Path;
use cargo::Config;
use cargo::core::SourceId;
use codegen_utils::{copy_dir, get_build_project_dir, project_directory};
use crate::{compile_in_target, download_cargo_crate, generate_knockoff_builder};

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
