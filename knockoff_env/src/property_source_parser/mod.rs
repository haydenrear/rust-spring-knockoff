use codegen_utils::walk::DirectoryWalker;
use knockoff_resource::{FilePathMatchingPatternResourceResolver, FileResource, PathMatchingPatternResourceResolver};
use crate::{EnvActiveProfileOrderings, TomlPropertySource};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("property_source_parser.rs");

pub trait PropertySourceParser {
}

pub struct TomlPropertySourceParser;

impl TomlPropertySourceParser {
    pub fn parse_property_sources(env_active_profile_orderings: EnvActiveProfileOrderings) -> Vec<TomlPropertySource> {
        let home_directory = project_directory!();
        env_active_profile_orderings.iter()
            .flat_map(|profile| Self::find_all_resources_for_profile(profile.0.as_str(), home_directory))
            .map(|p| TomlPropertySource::new(p))
            .collect()
    }

    pub fn find_all_resources_for_profile(profile_name: &str, home_directory: &str) -> Vec<FileResource>{
        let home_directory = format!("{}/.cargo", home_directory);
        let path_to_match = format!("{}/.*-{}.toml", home_directory, profile_name)
            .replace("/", "\\/");
        info!("Searching for config for profile name {}. {} is path to match and {} is directory searched",
            profile_name, path_to_match, home_directory);
        let matched = FilePathMatchingPatternResourceResolver::find_all_resources_matching_regexp(
            &path_to_match,
            &home_directory
        );
        matched
    }
}

#[test]
fn test_property_source_parser() {
    let matched = TomlPropertySourceParser::find_all_resources_for_profile("test", project_directory!());
    assert_eq!(matched.len(), 1);
}