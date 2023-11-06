use std::path::{Path, PathBuf};
use codegen_utils::get_project_base_dir;
use codegen_utils::walk::DirectoryWalker;
use web_framework_shared::{AntStringRequestMatcher, Matcher};
use crate::{Resource, ResourceUri};
use crate::file_resource::FileResource;

use knockoff_logging::*;
use lazy_static::lazy_static;
use crate::logger_lazy;
use std::sync::Mutex;
use regex::Regex;
import_logger!("resource_loader.rs");

pub trait ResourceLoader<ResourceTypeT: Resource> {
    fn get_resource(location: String) -> ResourceTypeT;
}

pub trait PathMatchingPatternResourceResolver<ResourceTypeT: Resource> {
    fn find_all_resources_matching_regexp(location_regexp: &str, starting_directory: &str) -> Vec<ResourceTypeT>;
    fn find_all_resources_matching_ant_matcher(location: String) -> Vec<FileResource>;

}

pub struct FilePathMatchingPatternResourceResolver;

impl PathMatchingPatternResourceResolver<FileResource> for FilePathMatchingPatternResourceResolver {
    /// Return resources matching pattern like /directory/next/*something
    fn find_all_resources_matching_regexp(location_regexp: &str, starting_directory: &str) -> Vec<FileResource> {
        info!("Found resource {:?}", location_regexp);
        Self::walk_file_resource_directories_regexp(location_regexp, starting_directory)
    }
    /// Return resources matching pattern like /directory/next/*something
    fn find_all_resources_matching_ant_matcher(location: String) -> Vec<FileResource> {
        let path_matcher = AntStringRequestMatcher::new(
            location.clone(), "/".to_string());

        if let Some(l) = location.find("*") {
            let starting_location_without_pattern = &location[..l-1];
            info!("Found resource {:?}", starting_location_without_pattern);
            Self::walk_file_resource_directories_ant_matcher(&path_matcher, starting_location_without_pattern)
        } else {
            Self::walk_file_resource_directories_ant_matcher(&path_matcher, &location)
        }
    }

}

impl FilePathMatchingPatternResourceResolver {
    fn walk_file_resource_directories_regexp(location_regexp: &str, starting_directory: &str) -> Vec<FileResource> {
        let location_regex = Regex::new(location_regexp).map_err(|e| {
            error!("Incompatible regexp: {}", location_regexp);
        });
        if location_regex.is_err() {
            return vec![];
        }
        let location_regex = location_regex.unwrap();
        Self::do_walk_directory_with_predicate(starting_directory, &|file| {
            location_regex.is_match(file.to_str().unwrap())
        })
    }



    fn walk_file_resource_directories_ant_matcher(
        path_matcher: &AntStringRequestMatcher,
        mut location_without_pattern: &str
    ) -> Vec<FileResource> {
        Self::do_walk_directory_with_predicate(location_without_pattern, &|file| {
            path_matcher.matches(file.to_str().unwrap())
        })
    }

    fn do_walk_directory_with_predicate(
        starting_directory: &str,
        predicate: &dyn Fn(&PathBuf) -> bool
    ) -> Vec<FileResource> {
        DirectoryWalker::walk_directories_matching_to_path(
            predicate,
            &|file| true,
            starting_directory,
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

#[test]
fn test_pattern_matcher() {
    let proj_base_dir = Path::new(&get_project_base_dir())
        .join("knockoff_env")
        .join("test_matcher");
    let mut base_dir = proj_base_dir.to_str().unwrap();
    let mut pattern_base_dir = format!("{}/.*", base_dir);
    pattern_base_dir = pattern_base_dir.replace("/", "\\/");
    println!("Running find all patterns with base dir {:?}", pattern_base_dir);
    let out = FilePathMatchingPatternResourceResolver::find_all_resources_matching_regexp(
        pattern_base_dir.as_str(),
        base_dir
    );
    println!("first: {:?}\n\n", out);
    assert_eq!(out.len(), 9);

    let proj_base_dir = Path::new(&get_project_base_dir())
        .join("knockoff_env")
        .join("test_matcher");
    let mut base_dir = proj_base_dir.to_str().unwrap();
    let mut pattern_base_dir = format!("{}/*", base_dir);
    assert!(pattern_base_dir.ends_with("test_matcher/*"));
    println!("Running find all patterns with base dir {:?}", pattern_base_dir);
    let out = FilePathMatchingPatternResourceResolver::find_all_resources_matching_ant_matcher(
        pattern_base_dir
    );
    assert_eq!(out.len(), 4);

    let proj_base_dir = Path::new(&get_project_base_dir())
        .join("knockoff_env")
        .join("test_matcher");
    let mut base_dir = proj_base_dir.to_str().unwrap();
    let mut pattern_base_dir = format!("{}/.*/six", base_dir);
    pattern_base_dir = pattern_base_dir.replace("/", "\\/");
    println!("Running find all patterns with base dir {:?}", pattern_base_dir);
    let out = FilePathMatchingPatternResourceResolver::find_all_resources_matching_regexp(
        pattern_base_dir.as_str(),
        base_dir
    );
    assert_eq!(out.len(), 2);

    let proj_base_dir = Path::new(&get_project_base_dir())
        .join("knockoff_env")
        .join("test_matcher");
    let mut base_dir = proj_base_dir.to_str().unwrap();
    let mut pattern_base_dir = format!("{}/*/six", base_dir);
    println!("Running find all patterns with base dir {:?}", pattern_base_dir);
    let out = FilePathMatchingPatternResourceResolver::find_all_resources_matching_ant_matcher(
        pattern_base_dir
    );
    assert_eq!(out.len(), 2);


}
