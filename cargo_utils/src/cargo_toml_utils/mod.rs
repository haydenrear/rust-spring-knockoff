use std::fs;
use std::fs::File;
use std::io::{Seek, Write};
use std::path::PathBuf;
use toml::Table;
use codegen_utils::FlatMapOptional;

use knockoff_logging::*;
use std::sync::Mutex;
use crate::logger_lazy;
import_logger!("cargo_toml_utils.rs");

pub mod update_toml;
pub use update_toml::*;

pub mod find_in_toml;
pub use find_in_toml::*;

pub fn get_package_toml(option: &Option<File>) -> Option<Table> {
    option.as_ref().flat_map_opt(|cargo_manifest_file| {
            let parsed_value = codegen_utils::parse::read_file_to_str(&cargo_manifest_file)
                .map_err(|e| {
                    error!("Error reading file: {:?}", e);
                });
            if parsed_value.is_err() {
                None
            } else {
                let out = toml::from_str::<Table>(parsed_value.unwrap().as_str()).ok();
                out
            }
        })
}

pub fn get_package_manifest_file(package_name: &str, target_dir: &PathBuf) -> Option<PathBuf> {
    let package = target_dir
        .join(package_name)
        .join("Cargo.toml");
    info!("Reading {:?}", package.to_str());
    if package.exists() {
        Some(package)
    } else {
        None
    }
}


pub fn do_on_dir(path: PathBuf, to_do: &dyn Fn(&Vec<String>)) {
    let _ = fs::read_dir(&path)
        .map(|dir| {
            let packages_ref = dir
                .into_iter()
                .flat_map(|i| i
                    .map_err(|e| {error!("Error when reading dir entry {:?}", e);})
                    .ok()
                    .flat_map_opt(|f| f.file_name().to_str().map(|s| s.to_string()))
                    .into_iter()
                )
                .filter(|path_to_add| path.join(path_to_add).join("Cargo.toml").exists())
                .collect::<Vec<String>>();
            info!("Running to do for {:?}", &packages_ref);
            to_do(&packages_ref);
        });
}

