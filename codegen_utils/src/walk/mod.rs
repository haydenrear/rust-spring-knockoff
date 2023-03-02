use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::fs;
use std::fs::DirEntry;
use syn::Pat;

#[cfg(test)]
pub mod test;

pub struct DirectoryWalker;

impl DirectoryWalker {

    pub fn walk_directory(module_name: &str, base_dir: &str) -> Option<PathBuf> {
        Self::find_dir_in_directory(&|file| file.contains(module_name), Some(base_dir))
    }

    pub fn find_dir_in_directory(search: &dyn Fn(&str) -> bool, base_dir_opt: Option<&str>) -> Option<PathBuf> {
        if base_dir_opt.is_none() {
            return None;
        }

        let base_dir = base_dir_opt.unwrap();

        OsString::from(base_dir.to_string()).to_str().map(|os_string_dir| {
            let dirs = fs::read_dir(Path::new(os_string_dir));

            dirs.map(|read_dir| {
                let dir_entries = read_dir
                    .filter(|d| d.is_ok())
                    .map(|d| d.unwrap())
                    .collect::<Vec<DirEntry>>();

                let found_dir = Self::get_dir(search, dir_entries);

                if found_dir.is_some() {
                    return found_dir;
                }

                None

            })
                .ok()
                .flatten()
                .map(|p| {
                    if p.is_dir() {
                        return p.join("mod.rs");
                    }
                    p
                })
        }).flatten()
    }

    pub fn get_dir(search: &dyn Fn(&str) -> bool, read_dir: Vec<DirEntry>) -> Option<PathBuf> {
        let mut path_bufs = read_dir.iter()
            .map(|dir_found| {
                if dir_found.path().as_os_str().to_str()
                    .filter(|dir_name| search(dir_name))
                    .is_some() {
                    return Some(dir_found.path().to_path_buf());
                }
                dir_found.path().to_str()
                    .map(|dir_str| Self::find_dir_in_directory(search, Some(dir_str)))
                    .flatten()
            })
            .flatten()
            .collect::<Vec<PathBuf>>();
        if path_bufs.len() == 1 {
            return Some(path_bufs.remove(0));
        } else if path_bufs.len() > 1 {
            return path_bufs.iter()
                .filter(|p| !p.is_dir())
                .map(|p| p.clone())
                .next()
        }
        None
    }
}