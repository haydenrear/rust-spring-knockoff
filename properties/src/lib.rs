use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{Error, Read};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use codegen_utils::program_src;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;

import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/program_parser.log"));

#[derive(Deserialize, Serialize, Debug)]
pub struct Properties {
    inner: HashMap<String, String>,
}

impl Properties {

    pub fn load_all_properties() -> Self {
        let mut props = Properties {inner: HashMap::new()};
        props.load(None);
        let knockoff_profiles = env::var("KNOCKOFF_PROFILES").unwrap();
        for profile_str in knockoff_profiles.split(",").map(|s| s.to_string()).collect::<Vec<String>>().into_iter().rev() {
            let profile = profile_str.trim();
            if profile == profile {
                props.load(Some(profile));
            }
        }

        for (k, v) in props.inner.iter() {
            env::set_var(k, v) ;
        }

        props
    }

    pub fn load(&mut self, profile: Option<&str>) {
        let project_dir = program_src!();
        let mut search_dir = PathBuf::from(project_dir).parent().unwrap().to_path_buf();

        let mut vars_map = env::vars()
            .map(|(key, value)| (key, value))
            .collect::<HashMap<String, String>>();

        let mut starting_vars: HashSet<String> = vars_map.keys().map(|k| k.clone()).collect::<HashSet<String>>();
        loop {
            let file_path = profile
                .map(|p| search_dir.join(Path::new(&format!(".cargo/knockoff-{}.toml", p))))
                .or(Some(Path::new(".cargo/knockoff.toml").to_path_buf()))
                .unwrap();

            if file_path.exists() {
                info!("Loading from {:?}", file_path);
                let mut file = File::open(file_path).unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();

                // Replace environment variables in content

                info!("Loaded {}", contents);

                let inner: toml::Value = toml::from_str(&contents).unwrap();
                inner.as_table().map(|t: &toml::Table| {
                    t.keys().for_each(|k| {
                        if starting_vars.contains(k) {
                            let value = t.get(k).unwrap().to_string();
                            info!("Adding key: {}, value: {}", k, value);
                            starting_vars.remove(k);
                            vars_map.insert(k.clone(), value.replace("\"", ""));
                        } else if !vars_map.contains_key(k) {
                            let value = t.get(k).unwrap().to_string();
                            info!("Adding key: {}, value: {}", k, value);
                            vars_map.insert(k.clone(), value.replace("\"", ""));
                        }
                    })
                });

            } else {
                info!("Could not load {}", file_path.to_str().unwrap());
            }

            if !search_dir.exists() || search_dir.to_path_buf() == Path::new(&env::var("PROJECT_BASE_DIRECTORY").unwrap()).to_path_buf() {
                info!("Stopping loading {:?}", search_dir);
                break;
            } else {
                search_dir = search_dir.parent().unwrap().to_path_buf();
            }
        }


        vars_map.iter().for_each(|(k, v)| {
            self.inner.insert(k.clone(), v.clone());
        });

    }

}

#[test]
pub fn test_properties() {
    let props = Properties::load_all_properties();
    assert!(props.inner.contains_key("hello"));
    assert_eq!(props.inner.get("hello").unwrap().to_string(), "goodbye".to_string());
    assert_eq!(props.inner.get("okay").unwrap().to_string(), "no way".to_string());
}