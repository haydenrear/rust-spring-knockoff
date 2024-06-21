use std::fs;
use std::fs::File;
use std::io::Write;
use codegen_utils::FlatMapOptional;

use knockoff_logging::*;
use std::sync::Mutex;
use toml::{Table, Value};
use cargo_options::CargoTomlOptions;
use crate::logger_lazy;
import_logger!("toml_writer.rs");

pub(crate) const version: &'static str = "0.1.5";

pub mod cargo_options;

pub struct CargoTomlWriter<'a> {
    pub options: &'a mut CargoTomlOptions<'a>
}


impl<'a> CargoTomlWriter<'a> {

    pub fn version(&self) -> &str {
        self.options.version.or(Some(version)).unwrap()
    }

    pub fn write_toml_overwrite_if_exists(&self) {
        self.options.target_path.as_ref()
            .flat_map_opt(|path| self.options.name.map(|name| (path, name)))
            .map(|(path, name)| {
                if !path.exists() {
                    let _ = fs::create_dir_all(path.join(name).parent().unwrap())
                        .map_err(*err::LOG_ERR);
                }

                let _ = File::options()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path.join(name).join("Cargo.toml"))
                    .map(|cargo_file| self.do_write_toml(name, cargo_file))
                    .map_err(*err::LOG_ERR);
            });
    }

    pub fn write_toml_if_not_exists(&self) {
        self.options.target_path.as_ref()
            .filter(|p| !p.exists())
            .flat_map_opt(|path| self.options.name.map(|name| (path, name)))
            .map(|(path, name)| {
                let created = fs::create_dir_all(path.join(name));
                let _ = File::create(path.join(name).join("Cargo.toml"))
                    .map(|cargo_file| self.do_write_toml(name, cargo_file))
                    .map_err(*err::LOG_ERR);
            })
            .or_else(|| {info!("Skipped generating toml"); None});
    }

    fn do_write_toml(&self, name: &str, mut cargo_file: File) {
        writeln!(&mut cargo_file, "{}", Self::get_starting_toml_prelude(name, self.version()).as_str()).unwrap();
        let mut deps = Table::new();
        self.options.dependencies.map(|bd| deps.insert("dependencies".to_string(), Value::Table(bd.clone())));
        self.options.build_dependencies.map(|bd| deps.insert("build-dependencies".to_string(), Value::Table(bd.clone())));
        writeln!(&mut cargo_file, "{}", deps).unwrap();

        if self.options.workspace {
            writeln!(&mut cargo_file, "[workspace]").unwrap();
        }
    }

    pub fn get_starting_toml_prelude(name: &str, version_value: &str) -> String {
        use std::fmt::Write;
        let mut prelude = "".to_string();
        let _ = writeln!(
            &mut prelude, r#"
[package]
name = "{}"
version = "{}"
edition = "2021"
"#,
            name,
            version_value);

        prelude

    }

}

