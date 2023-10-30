use std::{env, fs};
use std::fs::File;
use std::path::Path;
use std::io::Write;
use codegen_utils::env::get_project_base_build_dir;
use codegen_utils::project_directory;

pub struct CrateWriter;

#[test]
fn do_test() {
    CrateWriter::write_dummy_crate(concat!(project_directory!(), "target/test/Cargo.toml"), "test", "".to_string());
    assert!(Path::new(concat!(project_directory!(), "target/test/src/lib.rs")).exists());
}

impl CrateWriter {

    pub fn write_dummy_crate(cargo_file: &str, name: &str, dependencies: String) {
        TomlWriter::write_toml_if_not_exists(cargo_file, name, dependencies);
        LibWriter::write_lib_create_dirs(name);
    }
}

pub struct LibWriter;

impl LibWriter {
    pub fn write_lib_create_dirs(name: &str) {
        let mut out = String::default();

        out += project_directory!();
        out += "target/";
        out += name;
        out += "/src";

        fs::create_dir_all(&out);

        let path = Path::new(&out).join("lib.rs");

        if !path.exists()  {
            File::create(path);
        }
    }
}

pub struct TomlWriter;

impl TomlWriter {

    pub fn write_toml_if_not_exists(filepath: &str, name: &str, dependencies: String) {
        let path = Path::new(filepath);
        if path.exists() {
            return;
        }
        fs::create_dir_all(filepath.replace("Cargo.toml", ""));
        let mut cargo_file = File::create(path).unwrap();
        writeln!(&mut cargo_file, "{}", Self::get_starting_toml_prelude(name).as_str()).unwrap();
        writeln!(&mut cargo_file, "[dependencies]").unwrap();
        writeln!(&mut cargo_file, "{}", dependencies).unwrap();
    }

    pub fn get_starting_toml_prelude(name: &str) -> String {
        use std::fmt::Write;
        let mut prelude = "".to_string();

        writeln!(&mut prelude,
"[package]
name = \"{}\"
version = \"0.1.5\"
edition = \"2021\"
", name);
        prelude

    }

}
