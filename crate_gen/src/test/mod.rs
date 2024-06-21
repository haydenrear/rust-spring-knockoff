use std::fs::File;
use std::path::Path;
use toml::{Table, Value};
use codegen_utils::project_directory;
use crate::{CargoTomlWriter, CrateWriter};
use crate::toml_writer::cargo_options::CargoTomlOptions;

#[test]
fn try_build_create() {
    let doesnt_exist = Path::new(project_directory!()).join("crate_gen").join("test_resources");
    let _ = std::fs::remove_file(&doesnt_exist.join("Cargo.toml"));
    let mut opened = CargoTomlOptions::open_options();
    let options = opened
        .workspace(true)
        .name("first")
        .version("0.1.5")
        .target_path(doesnt_exist);

    let writer = CargoTomlWriter { options };

    writer.write_toml_if_not_exists();

    let does_exist = Path::new(project_directory!()).join("crate_gen").join("test_resources");
    let to_clear = Path::new(project_directory!()).join("crate_gen").join("test_resources").join("second").join("Cargo.toml");
    let _ = File::options().write(true).truncate(true) .open(&to_clear);
    let read = codegen_utils::read_dir_to_file(&to_clear).unwrap();
    assert_eq!(read, "".to_string());
    let mut opened = CargoTomlOptions::open_options();
    let options = opened
        .workspace(true)
        .name("second")
        .version("0.1.5")
        .target_path(does_exist);

    let writer = CargoTomlWriter { options };

    writer.write_toml_overwrite_if_exists();

    let read = codegen_utils::read_dir_to_file(&to_clear).unwrap();
    assert_ne!(read, "".to_string());
}

#[test]
fn do_test() {
    let target_dir = Path::new(project_directory!()).join("crate_gen").join("test_resources").join("test_lib_writer");
    let _ = std::fs::remove_dir_all(&target_dir);
    let _ = std::fs::create_dir_all(&target_dir);
    let value = Value::Table(vec![("name".to_string(), Value::String("hello".to_string())), ("version".to_string(), Value::String("0.1.5".to_string()))].into_iter().collect());
    CrateWriter::write_dependency_agg_crate("test", "0.1.5", &target_dir, Table::from(vec![("hello".to_string(), value)].into_iter().collect()));
    assert!(target_dir.join("test").exists());
    let opened = cargo_utils::get_package_toml(&Some(File::options().read(true).open(target_dir.join("test").join("Cargo.toml")).unwrap()));
    assert!(opened.is_some());
    let map = opened.unwrap();
    let deps = map.get("dependencies").unwrap().as_table();
    deps.as_ref().unwrap().keys().for_each(|k| println!("{}", k));
    assert!(deps.unwrap().get("hello").is_some());
}
