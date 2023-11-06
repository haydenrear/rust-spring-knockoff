use std::fmt::{Debug, Display};
use std::fs::File;
use std::path::PathBuf;
use enum_fields::EnumFields;
use http::Uri;
use web_framework_shared::Matcher;

use knockoff_logging::*;
use lazy_static::lazy_static;
use codegen_utils::project_directory;
use std::sync::Mutex;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/knockoff_resource.log"));

mod resource_loader;
pub use resource_loader::*;
mod file_resource;
pub use file_resource::*;

pub enum PathType {
    Relative, Absolute
}

#[derive(EnumFields)]
pub enum ResourceUri {
    File {
        path: PathBuf
    },
    Web {
        url: Uri
    }
}

pub trait Resource {
    fn get_file(&mut self) -> Option<&mut File>;
    fn get_uri(&self) -> &ResourceUri;
    fn get_content_as_str(&mut self) -> Result<String, std::io::Error>;
    fn get_content_as_bytes<'a>(&'a mut self, bytes_out: &'a mut [u8]) -> Result<&'a mut [u8], std::io::Error>;
    fn exists(&self) -> bool;
}

