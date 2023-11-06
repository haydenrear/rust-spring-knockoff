use std::path::Path;
use codegen_utils::parse::read_path_to_str;
use toml::{Table, Value};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use knockoff_helper::project_directory;
use crate::{EnvironmentProfiles, logger_lazy};
import_logger!("env_profile_ordering.rs");


pub struct ProfileOrderingParser;

impl ProfileOrderingParser {
    pub fn parse_profile_ordering() -> EnvironmentProfiles {
        let home = project_directory!();
        let config_toml = Path::new(home).join(".cargo").join("config.toml");
        if config_toml.exists() {
            read_path_to_str(&config_toml)
                .map(|read_str| toml::from_str::<toml::Table>(&read_str)
                    .map_err(|e| {
                        error!("Error reading config.toml into table {:?}.", e);
                    }).ok()
                )
                .map_err(|e| {
                    error!("Error reading config.toml file {:?}.", e);
                })
                .ok()
                .flatten()
                .map(|toml_table| {
                    Self::parse_env_profiles_from_toml_table(toml_table)
                })
                .flatten()
                .unwrap()
        } else {
            EnvironmentProfiles::default()
        }
    }

    fn parse_env_profiles_from_toml_table(toml_table: Table) -> Option<EnvironmentProfiles> {
        toml_table.get("knockoff_profiles")
            .map(|v| {
                Self::de_parse_env_profiles(v)
            })
            .flatten()
            .or_else(|| {
                info!("User did not provide profile information.");
                Some(EnvironmentProfiles::default())
            })
    }

    fn de_parse_env_profiles(v: &Value) -> Option<EnvironmentProfiles> {
        toml::to_string(v)
            .map_err(|e| {
                error!("Error parsing toml back to string to parse env profiles.");
            })
            .map(|v| {
                toml::from_str::<EnvironmentProfiles>(&v)
                    .map_err(|e| {
                        error!("Error parsing EnvironmentProfiles from {:?}", &v);
                    })
                    .ok()
            })
            .ok()
            .flatten()
    }
}
