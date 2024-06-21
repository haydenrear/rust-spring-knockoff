use std::path::PathBuf;
use toml::{Table, Value};
use std::fs::File;
use knockoff_logging::{error, info};
use std::io::{Seek, SeekFrom, Write};
use codegen_utils::FlatMapOptional;

use knockoff_logging::*;
use std::sync::Mutex;
use crate::logger_lazy;
import_logger!("update_toml.rs");

pub enum TomlUpdates<'a> {
    AddKeyToPackage(String, String),
    ChangeKeyPackage(String, String),
    AddPathToPackage(String, String),
    RemoveKeyFromBuildDependencies(String, &'a Vec<String>),
    RemoveKeyFromDependencies(String, &'a Vec<String>),
    AddKeyToBuildDependencies {
        packages: &'a Vec<String>,
        key: String,
        value: String
    },
    AddKeyToDependencies {
        packages: &'a Vec<String>,
        key: String,
        value: String
    },
    AddDependency {
        name: &'a str,
        path: Option<&'a str>,
        version: Option<&'a str>
    }
}

pub struct UpdateToml<'a> {
    updates: &'a Vec<TomlUpdates<'a>>,
    package: &'a str,
    target: &'a PathBuf,
    paths: &'a Vec<String>,
    pub toml_table: Option<Table>
}


impl<'a> UpdateToml<'a> {
    pub fn do_update_toml_from(updates: &'a Vec<TomlUpdates>, package: &'a str,
                               target: &'a PathBuf, paths: &'a Vec<String>) -> UpdateToml<'a> {
        let mut manifest_file = crate::get_package_manifest_file(package, target)
            .as_mut()
            .flat_map_opt(|p| File::options().write(true).read(true).open(p).map_err(|e| { info!("{:?}", e); }).ok());
        let package_toml = crate::get_package_toml(&manifest_file);

        info!("Retrieved package toml.");

        let mut u = Self {
            updates,
            target,
            package,
            paths,
            toml_table: package_toml,
        };

        u.do_update_toml(manifest_file);

        u
    }

    pub fn update_paths(
        package: &str,
        toml_table: &mut Table,
        updates: &Vec<TomlUpdates>
    ) {
        updates.iter().for_each(|toml_update| {
            if let TomlUpdates::RemoveKeyFromBuildDependencies(key, packages) = toml_update {
                toml_table.get_mut("build-dependencies").as_mut()
                    .map(|deps| Self::do_on_table_mut(packages, deps, package, &mut |t| t.remove(key)));
            }
            if let TomlUpdates::RemoveKeyFromDependencies(key, packages) = toml_update {
                toml_table.get_mut("dependencies").as_mut()
                    .map(|deps| Self::do_on_table_mut(packages, deps, package, &mut |t| t.remove(key)));
            }
            if let TomlUpdates::AddKeyToBuildDependencies{ packages, key, value } = toml_update {
                toml_table.get_mut("build-dependencies").as_mut()
                    .map(|deps| Self::do_on_table_mut(packages, deps, package, &mut |t| t.insert(key.to_string(), Value::String(value.to_string()))));
            }
            if let TomlUpdates::AddKeyToDependencies{ packages, key, value } = toml_update {
                toml_table.get_mut("dependencies").as_mut()
                    .map(|deps| Self::do_on_table_mut(packages, deps, package, &mut |t| t.insert(key.to_string(), Value::String(value.to_string()))));
            }
        });

    }

    pub fn do_on_dependency_block(toml_table: &mut Table, toml_update: &TomlUpdates) {
        if let TomlUpdates::RemoveKeyFromBuildDependencies(key, packages) = toml_update {
            packages.iter()
                .for_each(|p| Self::do_on_dep_table(toml_table, p, &mut |t| t.remove(key)));
        }
        if let TomlUpdates::RemoveKeyFromDependencies(key, packages) = toml_update {
            packages.iter()
                .for_each(|p| Self::do_on_dep_table(toml_table, p, &mut |t| t.remove(key)));
        }
        if let TomlUpdates::AddKeyToBuildDependencies{ packages, key, value } = toml_update {
            packages.iter()
                .for_each(|p| Self::do_on_dep_table(toml_table, p, &mut |t| t.insert(key.to_string(), Value::String(value.to_string()))));
        }
        if let TomlUpdates::AddKeyToDependencies{ packages, key, value } = toml_update {
            packages.iter()
                .for_each(|p| Self::do_on_dep_table(toml_table, p, &mut |t| t.insert(key.to_string(), Value::String(value.to_string()))));
        }
        if let TomlUpdates::AddPathToPackage(_, package) = toml_update {
            Self::insert_path_to_dependency_from_table(toml_table, package)
        }
        if let TomlUpdates::AddDependency { version, path, name } = toml_update {
            Self::do_on_dep_table(toml_table, name, &mut |table_to_update| {
                let mut dep_table = Table::new();
                version.map(|v| dep_table.insert("version".to_string(), Value::String(v.to_string())));
                path.map(|v| dep_table.insert("path".to_string(), Value::String(v.to_string())));
                table_to_update.insert(name.to_string(), Value::Table(dep_table))
            });
        }
    }


    fn do_on_table_mut(packages: &Vec<String>, deps: &mut Value, package: &str, to_do: &mut dyn FnMut(&mut Table) -> Option<Value>) {
        for other_package in packages.iter() {
            if package == *other_package {
                continue
            }
            Self::do_on_dep(deps, other_package, to_do);
        }
    }

    fn do_on_dep_table(deps: &mut Table,
                       other_package: &str,
                       to_do: &mut dyn FnMut(&mut Table) -> Option<Value>) {
        if !deps.contains_key(other_package) {
            to_do(deps);
        }
    }

    fn do_on_dep(deps: &mut Value,
                 other_package: &str,
                 to_do: &mut dyn FnMut(&mut Table) -> Option<Value>) {
        deps.as_table_mut()
            .map(|d| Self::do_on_dep_table(d, other_package, to_do));
    }

    pub fn do_update_toml(&mut self, mut manifest_file: Option<File>) {
        self.toml_table.as_mut()
            .map(|pkg| {
                Self::update_paths(self.package, pkg, self.updates);
                self.updates.iter().for_each(|t| {
                    match t {
                        TomlUpdates::AddKeyToPackage(k, v) => Self::do_on_pkg(pkg, &mut |t| t.insert(k.to_string(), Value::String(v.to_string()))),
                        TomlUpdates::ChangeKeyPackage(k, v) => Self::do_on_pkg(pkg, &mut |t| {
                            t.remove(k);
                            t.insert(k.to_string(), Value::String(v.to_string()))
                        }),
                        TomlUpdates::AddPathToPackage(_, package) => {
                            Self::insert_path(pkg, package, "build-dependencies");
                            Self::insert_path(pkg, package, "dependencies");
                        }
                        _ => {}
                    }
                });

                Self::write_cargo_toml(pkg, manifest_file.unwrap());
            });
    }


    fn write_cargo_toml(pkg: &mut Table, mut out_package: File) {
        let _ = out_package.seek(SeekFrom::Start(0));
        let _ = out_package.set_len(0);

        let _ = toml::to_string(pkg).map(|manifest_written| {
            info!("Writing manifest {:?} to {:?}", manifest_written, out_package);
            let _ = out_package.write_all(manifest_written.as_bytes())
                .map_err(|e| { error!("Error writing manifest {:?}", e); });
            if !manifest_written.contains("[workspace]") {
                let _ = out_package
                    .write("[workspace]".to_string().as_bytes())
                    .map_err(|e| { error!("Error writing manifest!"); });
            }
        });
    }

    pub fn insert_path_to_dependency(dep_block: &mut Value, package: &str) {
        Self::do_on_dep(dep_block, package,
                        &mut |t| t.insert("path".to_string(), Value::String(format!("../{}", package))));
    }

    pub fn insert_path_to_dependency_from_table(dep_block: &mut Table, package: &str) {
        Self::do_on_dep_table(dep_block, package,
                        &mut |t| t.insert("path".to_string(), Value::String(format!("../{}", package))));
    }

    fn do_on_pkg(pkg: &mut Table, to_do: &mut dyn FnMut(&mut Table) -> Option<Value>) {
        Self::do_on_key("package", pkg, to_do);
    }

    fn do_on_key(key: &str, pkg: &mut Table, to_do: &mut dyn FnMut(&mut Table) -> Option<Value>) {
        pkg.get_mut(key)
            .flat_map_opt(|pkg| pkg.as_table_mut())
            .map(|package_table| to_do(package_table));
    }

    fn insert_path(mut pkg: &mut Table, package: &String, package_ty: &str) {
        pkg.get_mut(package_ty)
            .filter(|v| v.as_table().map(|t| t.contains_key(package)).or(Some(false)).unwrap())
            .map(|v| Self::insert_path_to_dependency(v, package));
    }

}
