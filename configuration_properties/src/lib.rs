use std::collections::HashMap;
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;
use enum_fields::EnumFields;
use http::Uri;
use codegen_utils::parse::{open_file_from_path, read_file_to_bytes, read_file_to_str, read_path_to_str};
use codegen_utils::walk::DirectoryWalker;
use web_framework_shared::AntStringRequestMatcher;

pub struct Priority(usize);
pub struct EnvProfile(String);

pub trait ConfigurationPropertiesParser {
    fn input_envs(names: Vec<String>, project_dir: String) -> HashMap<String, String>;
}


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
    fn get_file(&mut self) -> &mut File;
    fn get_uri(&self) -> &ResourceUri;
    fn get_content_as_str(&mut self) -> Result<String, std::io::Error>;
    fn get_content_as_bytes(&mut self) -> Result<&[u8], std::io::Error>;
    fn exists(&self) -> bool;
}

pub trait ResourceLoader<ResourceTypeT: Resource> {
    fn get_resource(location: String) -> ResourceTypeT;
}

pub struct FileResource {
    file: Option<File>,
    uri: ResourceUri
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

    fn get_content_as_bytes(&mut self) -> Result<&[u8], std::io::Error>{
        if self.exists() {
            self.get_file().map(|f| {
               read_file_to_bytes(f)
            }).unwrap()
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

pub trait PathMatchingPatternResourceResolver<ResourceTypeT: Resource> {
    fn find_all_resources_matching(location: String) -> Vec<ResourceTypeT>;
}

pub struct FilePathMatchingPatternResourceResolver;

impl PathMatchingPatternResourceResolver<FileResource> for FilePathMatchingPatternResourceResolver {
    /// Return resources matching pattern like /directory/next/*something
    fn find_all_resources_matching(location: String) -> Vec<FileResource> {
        let path_matcher = AntStringRequestMatcher::new(location, "/".to_string());
        env::var("PROJECT_BASE_DIRECTORY").map(|proj| {
            DirectoryWalker::walk_find_mod()
        })
    }
}

pub struct TomlConfigurationPropertiesParser;

