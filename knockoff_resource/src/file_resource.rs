use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use codegen_utils::parse::{open_file_from_path, read_file_to_bytes, read_file_to_str};
use std::io::{Error, ErrorKind};
use crate::{Resource, ResourceUri};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("file_resource.rs");

pub struct FileResource {
    pub(crate) file: Option<File>,
    pub(crate) uri: ResourceUri
}

impl Debug for FileResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("FileResource: {:?}", self.uri.path()).as_str())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FileNotExistent;

impl Display for FileNotExistent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <FileNotExistent as Debug>::fmt(self, f)
    }
}

impl std::error::Error for FileNotExistent {
}

impl Resource for FileResource {
    fn get_file(&mut self) -> Option<&mut File> {
        if self.file.is_some() {
            self.file.as_mut()
        } else if self.exists() {
            let file = self.uri.path().map(|p| open_file_from_path(p).ok())
                .flatten();
            self.file = file;
            self.file.as_mut()
        } else {
            None
        }
    }

    fn get_uri(&self) -> &ResourceUri {
        &self.uri
    }

    fn get_content_as_str(&mut self) -> Result<String, std::io::Error> {
        if self.exists() {
            self.get_file().map(|f| {
                read_file_to_str(f)
            }).unwrap()
        } else {
            Err(Error::new(ErrorKind::NotFound, FileNotExistent {}))
        }
    }

    fn get_content_as_bytes<'a>(&'a mut self, bytes_out: &'a mut [u8]) -> Result<&'a mut [u8], std::io::Error>{
        if self.exists() {
            self.get_file().map(|f| read_file_to_bytes(f, bytes_out))
                .unwrap()
        } else {
            Err(Error::new(ErrorKind::NotFound, FileNotExistent {}))
        }
    }

    fn exists(&self) -> bool {
        if self.file.is_some() {
            true
        } else {
            self.uri.path().as_ref().map(|u| u.exists())
                .or(Some(false))
                .unwrap()
        }
    }
}

