use std::collections::HashMap;
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};
use enum_fields::EnumFields;
use http::Uri;
use codegen_utils::get_project_base_dir;
use codegen_utils::parse::{open_file_from_path, read_file_to_bytes, read_file_to_str, read_path_to_str};
use codegen_utils::walk::DirectoryWalker;
use web_framework_shared::{AntStringRequestMatcher, Matcher};

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
    fn get_file(&mut self) -> Option<&mut File>;
    fn get_uri(&self) -> &ResourceUri;
    fn get_content_as_str(&mut self) -> Result<String, std::io::Error>;
    fn get_content_as_bytes<'a>(&'a mut self, bytes_out: &'a mut [u8]) -> Result<&'a mut [u8], std::io::Error>;
    fn exists(&self) -> bool;
}

pub trait ResourceLoader<ResourceTypeT: Resource> {
    fn get_resource(location: String) -> ResourceTypeT;
}


pub struct FileResource {
    file: Option<File>,
    uri: ResourceUri
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

pub trait PathMatchingPatternResourceResolver<ResourceTypeT: Resource> {
    fn find_all_resources_matching(location: String) -> Vec<ResourceTypeT>;
}

pub struct FilePathMatchingPatternResourceResolver;

impl PathMatchingPatternResourceResolver<FileResource> for FilePathMatchingPatternResourceResolver {
    /// Return resources matching pattern like /directory/next/*something
    fn find_all_resources_matching(location: String) -> Vec<FileResource> {
        let path_matcher = AntStringRequestMatcher::new(
            location.clone(), "/".to_string());

        if let Some(l) = location.find("*") {
            let location_without_pattern = &location[..l-1];
            println!("Found resource {:?}", location_without_pattern);
            Self::walk_file_resource_directories(&path_matcher, location_without_pattern)
        } else {
            Self::walk_file_resource_directories(&path_matcher, &location)
        }
    }
}

impl FilePathMatchingPatternResourceResolver {
    fn walk_file_resource_directories(path_matcher: &AntStringRequestMatcher, mut location_without_pattern: &str) -> Vec<FileResource> {
        DirectoryWalker::walk_directories_matching_to_path(
                &|file| {
                    path_matcher.matches(file.to_str().unwrap())
                },
                &|file| true,
                location_without_pattern,
            )
            .into_iter()
            .map(|path| {
                FileResource {
                    file: None,
                    uri: ResourceUri::File {
                        path,
                    },
                }
            }).collect()
    }
}

pub struct TomlConfigurationPropertiesParser;

#[test]
fn test_pattern_matcher() {
    let proj_base_dir = Path::new(&get_project_base_dir())
        .join("configuration_properties")
        .join("test_matcher");
    let mut base_dir = proj_base_dir.to_str().unwrap();
    let mut pattern_base_dir = format!("{}/**", base_dir);
    assert!(pattern_base_dir.ends_with("test_matcher/**"));
    println!("Running find all patterns with base dir {:?}", pattern_base_dir);
    let out = FilePathMatchingPatternResourceResolver::find_all_resources_matching(
        pattern_base_dir
    );
    println!("first: {:?}\n\n", out);
    assert_eq!(out.len(), 9);

    let proj_base_dir = Path::new(&get_project_base_dir())
        .join("configuration_properties")
        .join("test_matcher");
    let mut base_dir = proj_base_dir.to_str().unwrap();
    let mut pattern_base_dir = format!("{}/*", base_dir);
    assert!(pattern_base_dir.ends_with("test_matcher/*"));
    println!("Running find all patterns with base dir {:?}", pattern_base_dir);
    let out = FilePathMatchingPatternResourceResolver::find_all_resources_matching(
        pattern_base_dir
    );
    assert_eq!(out.len(), 4);

    let proj_base_dir = Path::new(&get_project_base_dir())
        .join("configuration_properties")
        .join("test_matcher")
        .join("*");
    let mut base_dir = proj_base_dir.to_str().unwrap();
    let mut pattern_base_dir = format!("{}/six", base_dir);
    assert!(pattern_base_dir.ends_with("test_matcher/*/six"));
    println!("Running find all patterns with base dir {:?}", pattern_base_dir);
    let out = FilePathMatchingPatternResourceResolver::find_all_resources_matching(
        pattern_base_dir
    );
    assert_eq!(out.len(), 2);

}

