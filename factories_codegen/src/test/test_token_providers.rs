use std::fs;
use std::path::{Path, PathBuf};

use toml::Value;

use codegen_utils::project_directory;
use codegen_utils::walk::DirectoryWalker;
use crate_gen::{get_key, SearchType, SearchTypeError};

use crate::factories_parser::{FactoriesParser, Phase};

#[test]
fn test_parse_factories() {
    fs::remove_dir_all(test_resources().join("out"));

    let buf = get_test();

    FactoriesParser::write_phase(&"0.1.2".to_string(), &buf.to_str().unwrap().clone().to_string(), &test_resources().join("out").to_str().unwrap().to_string(), &Phase::Providers);
    FactoriesParser::write_phase(&"0.1.2".to_string(), &buf.to_str().unwrap().clone().to_string(), &test_resources().join("out").to_str().unwrap().to_string(), &Phase::PreCompile);
    FactoriesParser::write_phase(&"0.1.2".to_string(), &buf.to_str().unwrap().clone().to_string(), &test_resources().join("out").to_str().unwrap().to_string(), &Phase::DFactory);

    let found = DirectoryWalker::find_any_files_matching_file_name(&|file_name| file_name == "Cargo.toml", &test_resources().join("out"));

    let path_buf = found.get(test_resources().join("out").join("knockoff_precompile_gen").join("Cargo.toml").to_str().unwrap());
    assert!(path_buf.is_some());

    let toml_found: Value = toml::from_str(&codegen_utils::io_utils::read_dir_to_file(path_buf.unwrap()).unwrap()).unwrap();

    let found_key = get_key(vec![
        SearchType::FieldKey("dependencies".to_string()),
        SearchType::FieldKey("proc-macro2".to_string())
    ], &toml_found).map(|v| v.to_string());
    assert!(matches!(Ok::<String, SearchTypeError>("\"1.0\"".to_string()), found_key));

    let found_key = get_key(vec![
        SearchType::FieldKey("dependencies".to_string()),
        SearchType::FieldKey("knockoff_precompile_genone".to_string())
    ], &toml_found).map(|v| v.to_string());
    assert!(matches!(Ok::<String, SearchTypeError>, found_key));

    let found_key = get_key(vec![
        SearchType::FieldKey("package".to_string()),
        SearchType::FieldKey("version".to_string())
    ], &toml_found).map(|v| v.to_string());
    assert!(matches!(Ok::<String, SearchTypeError>("\"0.1.2\"".to_string()), found_key));
}

fn get_test() -> PathBuf {
    let buf = test_resources();
    let f = buf.join("knockoff_factories.toml");
    f
}

fn test_resources() -> PathBuf {
    let buf = Path::new(project_directory!()).join("factories_codegen").join("test_resources");
    buf
}

