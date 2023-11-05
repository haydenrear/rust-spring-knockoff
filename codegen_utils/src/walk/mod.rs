use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::fs;
use std::fs::DirEntry;
use syn::Pat;


use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::logger_lazy;
import_logger!("walk.rs");

#[cfg(test)]
pub mod test;

pub struct DirectoryWalker;

impl DirectoryWalker {

    pub fn walk_find_mod(module_name: &str, base_dir: &str) -> Vec<PathBuf> {
        let mut walk_dir;
        if module_name.ends_with(".rs") {
            walk_dir = Self::walk_dir(module_name.replace(".rs", "").as_str(), base_dir);
        } else {
            walk_dir = Self::walk_dir(module_name, base_dir);
        }
        walk_dir
            .into_iter()
            .map(|f| {
                info!("Searching {:?} for mod", f);
                f
            })
            .collect()
    }

    fn walk_dir(module_name: &str, base_dir: &str) -> Vec<PathBuf> {
        let mut out_bufs = vec![];
        Self::find_dir_in_directory(
            &|file| Self::is_mod(file, module_name),
            &|file| true,
            Some(base_dir), &mut out_bufs
        );
        out_bufs.iter()
            .map(|o| Path::new(o).to_path_buf())
            .collect()
    }

    fn is_mod(path_buf: &PathBuf, mod_name: &str) -> bool {
        if Self::is_parent_mod_name(path_buf, mod_name) && Self::file_name_equals(path_buf, "mod.rs") {
            info!("Found mod parent mod name: {}: {:?}", mod_name, path_buf);
            true
        } else {
            if Self::file_name_equals(path_buf, format!("{}.rs", mod_name).as_str()) {
                info!("Found mod file name equals: {}: {:?}", mod_name, path_buf);
                true
            } else {
                false
            }
        }

    }

    fn file_name_matches(path_buf: &PathBuf, to_match: &dyn Fn(&str) -> bool) -> bool {
        if path_buf.file_name().is_none() {
            false
        } else {
            path_buf.file_name().as_ref()
                .filter(|f| f.to_str()
                    .filter(|filename_to_match| to_match(filename_to_match))
                    .is_some()
                )
                .is_some()
        }
    }

    fn file_name_equals(path_buf: &PathBuf, to_match: &str) -> bool {
        if path_buf.file_name().is_none() {
            false
        } else {
            info!("Testing if file name {} is {:?}", to_match, path_buf);
            if path_buf.file_name()
                .filter(|f| f.to_str()
                    .filter(|&parent_dir_name| parent_dir_name == to_match)
                    .is_some()
                )
                .is_some() {
                info!("Found file name {}", to_match);
                true
            } else {
                false
            }
        }
    }

    fn is_parent_mod_name(path: &PathBuf, mod_name: &str) -> bool {
        path.parent().filter(|parent_path| {
            if !parent_path.is_dir() {
                false
            } else {
                Self::file_name_equals(path, mod_name)
            }
        }).is_some()
    }

    pub fn walk_directories_matching(
        search_file: &dyn Fn(&PathBuf) -> bool,
        search_dir: &dyn Fn(&PathBuf) -> bool,
        base_dir: &str
    ) -> Vec<String> {
        let mut out_paths = vec![];
        Self::find_dir_in_directory(search_file,
                                    search_dir,
                                    Some(base_dir), &mut out_paths);
        out_paths
    }

    pub fn find_dir_in_directory(
        search_add_file: &dyn Fn(&PathBuf) -> bool,
        continue_search_directory: &dyn Fn(&PathBuf) -> bool,
        base_dir_opt: Option<&str>,
        path_bufs: &mut Vec<String>
    ) {
        if base_dir_opt.is_none() {
            return
        }

        let base_dir = base_dir_opt.unwrap();

        OsString::from(base_dir.to_string()).to_str()
            .into_iter()
            .for_each(|os_string_dir| {
                info!("Searching {}", os_string_dir);
                let dirs = fs::read_dir(Path::new(os_string_dir))
                    .map_err(|e| {
                        error!("Failed to read dirs {:?}", &e);
                    });

                let _ = dirs.map(|read_dir| {
                    info!("Searching next dir: {:?}", read_dir);
                    let dir_entries = read_dir
                        .filter(|d| d.is_ok())
                        .map(|d| d.unwrap())
                        .collect::<Vec<DirEntry>>();

                    Self::get_dir(search_add_file, continue_search_directory, dir_entries, path_bufs)
                });
            });
    }

    pub fn get_dir(
        search_add: &dyn Fn(&PathBuf) -> bool,
        continue_search_directory: &dyn Fn(&PathBuf) -> bool,
        read_dir: Vec<DirEntry>,
        path_bufs: &mut Vec<String>
    ) {
        read_dir.iter()
            .for_each(|dir_found| {
                info!("Found next dir entry: {:?}", dir_found);
                if search_add(&dir_found.path()) {
                    path_bufs.push(dir_found.path().to_path_buf().to_str().unwrap().to_string());
                }
                if continue_search_directory(&dir_found.path()) {
                    dir_found.path().to_str()
                        .iter()
                        .for_each(|dir_str|
                            Self::find_dir_in_directory(
                                search_add,
                                continue_search_directory,
                                Some(dir_str),
                                path_bufs
                            )
                        );
                }
            });
    }
}