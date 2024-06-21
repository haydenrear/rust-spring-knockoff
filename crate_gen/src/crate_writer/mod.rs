use std::path::PathBuf;
use toml::Table;
use crate::{CargoTomlWriter, LibWriter};
use crate::cargo_options::CargoTomlOptions;

pub struct CrateWriter<'a> {
    toml: CargoTomlWriter<'a>,
    lib: LibWriter
}


impl<'a> CrateWriter<'a> {
    /// Writes a crate with the given dependencies and no code in the lib.rs.
    pub fn write_dependency_agg_crate(name: &str, version: &str, target_path: &PathBuf, dependencies: &Table) {
        LibWriter::write_empty_lib_rs(name, &target_path);
        Self::write_cargo_toml(name, target_path, dependencies, version);
    }

    /// Writes a crate with the given dependencies and code in the lib.rs.
    pub fn write_lib_rs_crate(name: &str, version: &str, target_path: &PathBuf, dependencies: &Table, code: &String) {
        let lib_writer = LibWriter::new(code);
        lib_writer.write_lib(name, target_path);
        Self::write_cargo_toml(name, target_path, dependencies, version);
    }

    fn write_cargo_toml(name: &str, target_path: &PathBuf, dependencies: &Table, version: &str) {
        let mut options = &mut CargoTomlOptions::open_options();
        let opt = CargoTomlWriter { options: options.target_path(target_path.clone()).name(name).version(version).dependencies(dependencies) };
        opt.write_toml_overwrite_if_exists();
    }
}
