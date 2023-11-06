use std::env;
use std::path::Path;
use factories_codegen::factories_parser::{FactoriesParser, FactoryStages};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use codegen_utils::{get_build_project_dir, get_project_base_build_dir, get_project_dir};
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/test_module_macro_codegen.log"));

#[test]
fn test_serialize() {
    let knockoff_factories = env::var("TEST_MODULE_MACRO_CODEGEN_KNOCKOFF_FACTORIES")
        .ok()
        .or(Some(get_project_dir("codegen_resources/knockoff_factories.toml")))
        .unwrap();
    let parsed = FactoriesParser::parse_factories_value::<FactoryStages>(knockoff_factories.as_str());
    let first_stage = parsed.as_ref().unwrap().stages.get("one");
    let factory_stage = first_stage.as_ref().unwrap();
    let p = factory_stage.get_providers();
    assert!(p.get("handler_mapping").as_ref().unwrap().dependency_data.as_ref().is_some());
    let token_provider = factory_stage.token_provider.as_ref().unwrap();
    println!("{:?} is parsed", parsed);
    assert!(parsed.as_ref().is_some());
    assert!(factory_stage.token_provider.as_ref().is_some());
    assert!(token_provider.values.as_ref().is_some());
    assert!(token_provider.values.as_ref().unwrap().get("handler_mapping").as_ref().unwrap().dependency_data.as_ref().is_some());
    assert!(factory_stage.dependencies.as_ref().is_some());
}
#[test]
fn test_write() {
    let knockoff_version = env::var("KNOCKOFF_VERSIONS")
        .or::<String>(Ok("0.1.5".into())).unwrap();
    let knockoff_factories = env::var("TEST_MODULE_MACRO_CODEGEN_KNOCKOFF_FACTORIES")
        .ok()
        .or(Some(get_project_dir("codegen_resources/knockoff_factories.toml")))
        .unwrap();

    let base_dir_path = env::var("TEST_MODULE_MACRO_CODEGEN_WORKDIR")
        .map(|t| Path::new(t.as_str()).to_path_buf())
        .ok()
        .unwrap();
    let out_dir_path = base_dir_path.join("target");
    let base_dir = base_dir_path.to_str().unwrap().to_string();
    let out_directory = out_dir_path.to_str().unwrap().to_string();

    info!("Found out directory: {:?}", out_directory);
    info!("Found base directory: {:?}", base_dir);

    FactoriesParser::write_phase(&knockoff_version, &knockoff_factories, &base_dir, &out_directory)
        .map(|stages| FactoriesParser::write_tokens_lib_rs(stages, &out_directory, &knockoff_version));

    assert!(out_dir_path.join("knockoff_providers_genone").exists());
    assert!(out_dir_path.join("knockoff_providers_genone").join("src").exists());
    assert!(out_dir_path.join("knockoff_providers_genone").join("src").join("lib.rs").exists());
    assert!(out_dir_path.join("knockoff_providers_genone").join("Cargo.toml").exists());

    assert!(out_dir_path.join("knockoff_providers_gen").exists());
    assert!(out_dir_path.join("knockoff_providers_gen").join("src").exists());
    assert!(out_dir_path.join("knockoff_providers_gen").join("src").join("lib.rs").exists());
    assert!(out_dir_path.join("knockoff_providers_gen").join("Cargo.toml").exists());

}