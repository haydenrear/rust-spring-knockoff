use std::{env, fs};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use toml::{Table, Value};
use proc_macro2::{Ident, Span};
use syn::parse_str;
use std::io::{Read, Write};
use codegen_utils::env::{get_build_project_dir, get_build_project_path, get_project_dir, get_project_path};
use crate_gen::TomlWriter;
use crate::parse_container_modifier::ParseContainerModifierProvider;
use crate::provider::{DelegatingProvider};
use crate::parse_provider::ParseProvider;
use crate::token_provider::TokenProvider;
use knockoff_logging::log_message;
use knockoff_logging::knockoff_logging::logging_facade::LoggingFacade;
use knockoff_logging::knockoff_logging::log_level::LogLevel;
use executors::common::Executor;
use serde::{Deserialize, Serialize};
use toml::map::Map;
use knockoff_logging::knockoff_logging::logger::Logger;
use knockoff_logging::knockoff_logging::default_logging::executor;
use knockoff_logging::knockoff_logging::default_logging::StandardLoggingFacade;

use crate::provider::ProviderProvider;
pub struct FactoriesParser;

macro_rules! providers {


    ($(($ty:ident, $factory_name:ident)),*) => {

        use proc_macro2::TokenStream;
        use quote::TokenStreamExt;


        #[derive(Serialize, Deserialize, Clone)]
        pub struct Factories {
            pub dependencies: Option<Value>,
            $(
                pub $factory_name: Option<HashMap<String, Provider>>,
            )*
        }

        impl DelegatingProvider for FactoriesParser {
            fn tokens() -> TokenStream {
                let mut ts = TokenStream::default();
                $(
                    ts.append_all($ty::tokens());
                )*
                ts
            }
        }

        $(
            impl DelegatingProvider for $ty {
                fn tokens() -> TokenStream {
                    FactoriesParser::parse_factories_value()
                        .map(|factories| {
                            let providers = factories.$factory_name
                                .iter()
                                .flat_map(|t| t.values())
                                .collect::<Vec<&Provider>>();
                            $ty::get_tokens(&providers)
                        })
                        .or(Some($ty::get_tokens(&vec![])))
                        .unwrap()
                }
            }
        )*


    }
}

providers!(
    (ParseProvider, parse_provider),
    (TokenProvider, token_provider),
    (ParseContainerModifierProvider, parse_container_modifier)
);

impl Factories {
    pub fn get_providers(&self) -> HashMap<String, Provider> {
        let mut provider_map = HashMap::new();
        self.insert_provider(&mut provider_map, &self.token_provider);
        self.insert_provider(&mut provider_map, &self.parse_provider);
        self.insert_provider(&mut provider_map, &self.parse_container_modifier);
        provider_map
    }

    fn insert_provider(&self, mut provider_map: &mut HashMap<String, Provider>, option: &Option<HashMap<String, Provider>>) {
        option.as_ref().map(|token_provider| {
            token_provider.iter().for_each(|t| {
                provider_map.insert(t.0.clone(), t.1.clone());
            });
        });
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Provider {
    pub provider_data: Option<ProviderData>,
    pub dependency_data: Option<Value>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProviderData {
    pub provider_path: Option<String>,
    pub provider_ident: Option<String>,
}

impl FactoriesParser {

    pub fn parse_factories_value() -> Option<Factories> {

        let knockoff_factories = env::var("KNOCKOFF_FACTORIES")
            .ok()
            .or(Some(get_project_dir("codegen_resources/knockoff_factories.toml")))
            .unwrap();

        File::open(knockoff_factories)
            .as_mut().map(|f| {
            let mut all_value = "".to_string();
            f.read_to_string(&mut all_value)
                .expect("Could not read factories.toml");
            all_value
        }).map(|all_value| {
            toml::from_str(all_value.as_str())
                .map_err(|err| {
                    println!("{}", err.to_string());
                })
                .ok()
        }).ok().flatten()
    }

    pub fn get_starting_toml_prelude() -> String {
        let mut prelude =
"[package]
name = \"knockoff_providers_gen\"
version = \"0.1.4\"
edition = \"2021\"
";
        prelude.to_string()
    }

    pub fn get_deps_and_providers(factories: Option<Factories>) -> (Option<Value>, HashMap<String, Provider>) {
        let deps = factories
            .as_ref()
            .map(|factories| {
                factories.dependencies.as_ref()
            })
            .flatten()
            .map(|d| {
                d.clone()
            });
        let providers = factories
            .as_ref()
            .map(|f| f.get_providers())
            .or(Some(HashMap::new()))
            .unwrap();
        (deps, providers)
    }

    pub fn write_cargo_toml(cargo_file: &str, base_dir: String) {
        let path = Path::new(cargo_file);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
        log_message!("Opening {}", &cargo_file);
        let mut cargo_file = File::create(path).unwrap();
        let cargo_str = Self::get_cargo_toml_string(&base_dir);
        cargo_file.write_all(cargo_str.as_bytes())
            .unwrap();
    }

    pub(crate) fn get_cargo_toml_string(base_dir: &String) -> String {

        use std::fmt::Write;

        let (mut knockoff_providers_dep, parsed_factories) = FactoriesParser::get_deps_and_providers(
            FactoriesParser::parse_factories_value()
        );
        let mut cargo_file = "".to_string();
        writeln!(&mut cargo_file, "{}", Self::get_starting_toml_prelude().as_str()).unwrap();
        knockoff_providers_dep.map(|mut dep| {
            let mut dep_table = Table::new();
            dep_table.insert("dependencies".to_string(), dep);
            Self::remove_paths_from_dependencies_table(&mut dep_table, base_dir);
            writeln!(&mut cargo_file, "{}", dep_table.to_string()).unwrap();
        });
        Self::write_provider_dependency_data(parsed_factories, &mut cargo_file, base_dir);
        writeln!(&mut cargo_file, "[workspace]")
            .unwrap();
        cargo_file
    }

    fn write_provider_dependency_data(parsed_factories: HashMap<String, Provider>, mut cargo_file: &mut String, base_dir: &String) {

        use std::fmt::Write;

        parsed_factories.iter().for_each(|p| {
            p.1.dependency_data.clone().as_mut().map(|dep_data| {
                let mut out_table = Table::default();
                let mut dep_table = Table::default();
                dep_table.insert(p.0.clone(), dep_data.clone());
                out_table.insert("dependencies".to_string(), Value::Table(dep_table));
                Self::remove_paths_from_dependencies_table(&mut out_table, base_dir);
                writeln!(&mut cargo_file, "{}", out_table.to_string()).unwrap();
            });
        });
    }

    fn remove_paths_from_dependencies_table(mut out_table: &mut Map<String, Value>, base_dir: &String) {
        // if the module_macro_lib library is in the project directory, then keep the ../path in the
        // Cargo.toml.
        if !Path::new(&base_dir).join("module_macro_lib").exists() {
            log_message!("Removing paths from knockoff_providers_gen Cargo.toml because not knockoff dev.");
            out_table.get_mut("dependencies")
                .map(|out| out.as_table_mut()
                    .map(|t| {
                        let keys = t.keys().map(|s| s.clone()).collect::<Vec<String>>();
                        keys.iter().for_each(|key| {
                            t.get_mut(key).unwrap().as_table_mut().map(|t| {
                                t.remove("path");
                            });
                        });
                    })
                );
        }
    }

    pub fn write_tokens_lib_rs(mut directory_tuple: String) {
        let lib_rs_file_path = Path::new(directory_tuple.as_str());

        fs::remove_file(lib_rs_file_path);

        File::create(lib_rs_file_path)
            .map(|mut lib_rs_file| Self::write_lib_rs(&mut lib_rs_file))
            .ok()
            .flatten()
            .or_else(|| {
                log_message!("Could not write to lib.rs file.");
                None
            });
    }

    fn write_lib_rs(mut lib_rs_file: &mut File) -> Option<()> {
        let parsed_factories = <FactoriesParser as DelegatingProvider>::tokens();
        writeln!(&mut lib_rs_file, "{}", parsed_factories.to_string().as_str())
            .ok()
    }


}