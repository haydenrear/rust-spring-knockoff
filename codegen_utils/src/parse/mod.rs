use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::logger_lazy;
import_logger!("parse.rs");

pub fn open_file(base_env: &str, lib_file: &str) -> Result<File, std::io::Error> {
    open_file_from_path(
        &Path::new(base_env)
            .join(lib_file)
    )
}

pub fn open_file_from_path(lib_file: &PathBuf) -> Result<File, std::io::Error> {
    File::open(lib_file)
}


pub fn read_file_to_str(file: &PathBuf) -> Result<String, std::io::Error> {
    File::open(file)
        .map(|mut f| {
            let mut out_str = String::default();
            let e = f.read_to_string(&mut out_str);
            let _ = e.map_err(|e| {
                error!("Error reading {:?} {:?}", file.to_str(), e);
            });
            out_str
        })
}