use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

use codegen_utils::project_directory;

use knockoff_logging::*;
use std::sync::Mutex;
use crate::toml_writer::cargo_options::CargoTomlOptions;
use crate::logger_lazy;
import_logger!("toml_writer.rs");

use crate::CrateWriter;

pub struct LibWriter {
    pub(crate) code: Option<String>
}

impl LibWriter {

    pub fn new(code: &String) -> Self {
        Self {
            code: Some(code.clone())
        }
    }

    pub fn write_empty_lib_rs(name: &str, target_path: &PathBuf) {
        let path = Self::get_lib_path(name, target_path);
        let _ = codegen_utils::io_utils::rewrite_file(path.as_path(), "".to_string())
            .map_err(*err::LOG_ERR);
    }

    fn get_lib_path(name: &str, target_path: &PathBuf) -> PathBuf {
        let path = target_path.join(name).join("src");
        let _ = fs::create_dir_all(&path);
        let path = path.join("lib.rs");
        path
    }

    pub fn write_lib(&self, name: &str, target_path: &PathBuf) {
        let empty_string = "".to_string();
        let to_write = self.code.as_ref().or(Some(&empty_string)).unwrap();
        let lib_path = Self::get_lib_path(name, target_path);
        let _ =  codegen_utils::io_utils::rewrite_file(&lib_path, to_write.to_string());
    }
}

