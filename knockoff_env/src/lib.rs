use std::collections::HashMap;
use std::fmt::{Debug, Display};
use web_framework_shared::Matcher;

use codegen_utils::project_directory;
use knockoff_logging::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/knockoff_env.log"));

mod profiles_parser;
pub use profiles_parser::*;
mod profile_priority;
pub use profile_priority::*;
mod environment;
pub use environment::*;
mod property_source;
pub use property_source::*;
mod property_source_parser;
pub use property_source_parser::*;


pub trait ConfigurationPropertiesParser {
    fn input_envs(names: Vec<EnvironmentProfiles>) -> HashMap<String, String>;
}


pub struct TomlConfigurationPropertiesParser;

impl ConfigurationPropertiesParser for TomlConfigurationPropertiesParser {
    fn input_envs(names: Vec<EnvironmentProfiles>) -> HashMap<String, String> {
        todo!()
    }
}

pub struct EnvironmentInitializer;

