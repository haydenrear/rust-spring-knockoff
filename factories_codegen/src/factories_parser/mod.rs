use std::collections::HashMap;
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::Path;
use std::sync::Mutex;

use executors::common::Executor;
use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use toml::{Table, Value};

use codegen_utils::{FlatMapOptional, FlatMapResult, project_directory};
use knockoff_helper::{project_directory_path};
use collection_util::IntoMultiMap;
use crate_gen::{CrateWriter, get_key, SearchType, SearchTypeError, TomlUpdates, UpdateToml};
use knockoff_logging::*;

use crate::factories_parser::factories::{Factories, FactoryPhases, FactoryStages};
use crate::logger_lazy;
use crate::provider::DelegatingProvider;
use crate::provider::ProviderProvider;

import_logger!("factories_parser.rs");

pub mod providers;
pub mod factories;

pub use factories::*;
pub use providers::*;

pub struct FactoriesParser {
    factories: Factories
}

pub trait DelegatingFactoriesParser {
    fn tokens(&self) -> TokenStream;
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug, Clone)]
pub enum Phase {
    #[serde(rename = "pre_compile")]
    PreCompile,
    #[serde(rename = "providers")]
    Providers,
    /// DFactory generates code that gets imported to import to generate mutation macro.
    /// When the user provides a dfactory, then a dfactory module gets created that then gets imported
    /// into the dfactory_dcodegen build.rs
    #[serde(rename = "dfactory")]
    DFactory,
    #[serde(rename = "all")]
    All
}

impl Display for Phase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Phase::PreCompile => f.write_str("pre_compile"),
            Phase::Providers => f.write_str("providers"),
            Phase::DFactory => f.write_str("dfactory"),
            Phase::All => f.write_str("all")

        }
    }
}

impl Phase {
    pub fn prefix(&self) -> &'static str {
        match self {
            Phase::PreCompile => "knockoff_precompile_gen",
            Phase::Providers => "knockoff_providers_gen",
            Phase::DFactory => "knockoff_dfactory_gen",
            _ => panic!("Not a valid!")
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Provider {
    pub provider_data: Option<ProviderData>,
    pub dependency_data: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProviderData {
    pub provider_path: Option<String>,
    pub provider_path_use_statement: Option<String>,
    pub provider_ident: Option<String>,
}

impl FactoriesParser {

    pub fn write_phase(version: &String, knockoff_factories: &String, out_directory: &String, phase: &Phase) {
        Self::parse_factories_value::<FactoryPhases>(knockoff_factories)
            .as_mut()
            .flat_map_opt(|f| {
                Self::add_stage_zero_default(phase, f);

                f.phase_deps.as_mut().map(|v| Self::add_all_to_table(v, Self::default_phase_deps(phase)));

                if f.phases.contains_key(phase) {
                    f.phases.remove(phase).map(|s| (false, s))
                } else {
                    info!("Did not find key: {:?}.", phase);
                    Some((true, FactoryStages {
                        stages: HashMap::new(),
                        gen_deps: f.phase_deps.as_ref().cloned().or(Some(Self::default_phase_deps(phase)))
                    }))
                }
            })
            .as_mut()
            .map(|(default, stages)| {

                let mut gen_deps = None;
                std::mem::swap(&mut gen_deps, &mut stages.gen_deps);

                if *default {
                    println!("Writing default crate for {}", &phase.to_string());
                    info!("Writing default crate for {}", &phase.to_string());
                    FactoriesParser::write_default_crates_for_phase(&out_directory, version, phase, gen_deps);
                } else {
                    gen_deps.as_mut()
                        .flat_map_opt(|dep| dep.as_table_mut())
                        .map(|t| Self::add_delegating_deps(stages.stages.keys().map(|s| s.as_str()).collect(), phase, t));

                    std::mem::swap(&mut gen_deps, &mut stages.gen_deps);
                    
                    println!("Writing factory crate for {}", &phase.to_string());
                    info!("Writing factory crate for {}", &phase.to_string());

                    FactoriesParser::write_factory_crate(stages, &out_directory, version, phase);
                }

                None::<bool>
            });
    }

    fn add_stage_zero_default(phase: &Phase, f: &mut FactoryPhases) {
        project_directory_path!().join("codegen_resources")
            .join("knockoff_default_factories.toml")
            .to_path_buf().to_str()
            .flat_map_opt(|knockoff_default_factories_path| Self::parse_factories_value::<FactoryPhases>(knockoff_default_factories_path))
            .map(|mut factories| {
                info!("Found factories {}", toml::to_string(&factories).unwrap());
                if !f.phases.contains_key(phase) {
                    Self::do_insert_zero_stage_phase(phase, f, &mut factories);
                } else {
                    Self::do_update_zero_stage_phase(phase, f, factories);
                }
            });
    }

    fn do_update_zero_stage_phase(phase: &Phase, f: &mut FactoryPhases, mut factories: FactoryPhases) {
        info!("Updating phase.");
        factories.phases.get_mut(phase)
            .flat_map_opt(|factory_stage_from| factory_stage_from.stages.remove("zero"))
            .map(|factories| f.phases.get_mut(phase)
                .map(|s| s.stages.insert("zero".to_string(), factories))
            );
    }

    fn do_insert_zero_stage_phase(phase: &Phase, f: &mut FactoryPhases, mut factories: &mut FactoryPhases) {
        factories.phases.get_mut(phase)
            .flat_map_opt(|factory_stage_from| {
                let mut gen_deps = None;
                std::mem::swap(&mut gen_deps, &mut factory_stage_from.gen_deps);
                factory_stage_from.stages.remove("zero")
                    .map(|f| (f, gen_deps))
            })
            .map(|(factories_to_add, gen_deps)| {
                let stages = FactoryStages {
                    stages: vec![("zero".to_string(), factories_to_add)].into_iter().collect(),
                    gen_deps
                };
                f.phases.insert(phase.clone(), stages)
            });
    }


    pub fn get_crate_name(stage_id: &str, phase: &Phase) -> String {
        format!("{}{}", phase.prefix(), stage_id)
    }

    pub fn write_default_crates_for_phase(
        out_directory: &String,
        version: &String,
        phase: &Phase,
        mut deps: Option<Value>) {

        let factories = Factories::create_providers_for_stages(vec!["one".to_string()], version, phase)
            .to_string();

        info!("Writing default rs for {} {:?} {}", out_directory, phase, factories);

        CrateWriter::write_lib_rs_crate(
            Self::get_crate_name(&"one".to_string(), phase).as_str(),
            version,
            &Path::new(out_directory).to_path_buf(),
            Self::default_stage_deps(phase, "one").as_table().unwrap(),
            &FactoriesParser::get_default_tokens().to_string()
        );

        let mut default_phase = Self::default_phase_deps(phase);
        deps.map(|t| Self::add_all_to_table(&mut default_phase, t));
        Self::add_dep_stage_crate(phase, default_phase.as_table_mut().unwrap(), "one");

        CrateWriter::write_lib_rs_crate(
            phase.prefix(),
            version,
            &Path::new(out_directory).to_path_buf(),
            default_phase.as_table().unwrap(),
            &factories
        );

    }

    pub fn write_factory_crate(
        factories_parser: &mut FactoryStages,
        out_directory: &String,
        version: &String,
        phase: &Phase
    ) {

        info!("Writing stages {:?} {:?}", phase, &factories_parser);
        let factories_parsed = Self::write_delegating_delegators(factories_parser, version, phase);

        factories_parser.gen_deps.as_mut().map(|v| Self::add_all_to_table(v, Self::default_phase_deps(&phase)));

        factories_parser.gen_deps.as_mut()
            .or(Some(&mut Self::default_phase_deps(&phase)))
            .as_mut()
            .flat_map_opt(|t| t.as_table_mut())
            .map(|deps| {

                factories_parser.stages.iter()
                    .for_each(|(stage_name, factory)| Self::add_dep_stage_crate(phase, deps, stage_name));
                
                info!("Doing writing of lib crate for stages {:?}\n{}", phase, &factories_parsed);

                CrateWriter::write_lib_rs_crate(
                    phase.prefix(),
                    version,
                    &Path::new(out_directory).to_path_buf(),
                    deps,
                    &factories_parsed
                );
            });

        for (stage_name, factory) in factories_parser.stages.iter_mut() {
            let lib_rs = Self::write_lib_rs(factory);
            info!("Doing writing of lib crate for phase {:?}: {}\n{}\n{}", phase, stage_name, toml::to_string(factory).unwrap(), &lib_rs);
            factory.dependencies.as_mut()
                .map(|v| Self::add_all_to_table(v, Self::default_stage_deps(phase, stage_name)));
            factory.dependencies.as_ref()
                .or(Some(&Self::default_stage_deps(phase, stage_name)))
                .flat_map_opt(|v| v.as_table())
                .map(|deps| {
                    CrateWriter::write_lib_rs_crate(
                        Self::get_crate_name(stage_name, phase).as_str(),
                        version,
                        &Path::new(out_directory).to_path_buf(),
                        deps,
                        &lib_rs
                    );
                });
        }
    }

    fn add_dep_stage_crate(phase: &Phase, deps: &mut Table, stage_name: &str) {
        let crate_name = Self::get_crate_name(stage_name, phase);
        info!("Adding dep stage crate {:?} to {}", &deps.to_string(), &crate_name);
        UpdateToml::do_on_dependency_block(
            deps,
            &TomlUpdates::AddDependency {
                name: &crate_name,
                path: Some(format!("../{}", &crate_name).as_str()),
                version: None
            });
    }

    pub fn parse_factories_value<T: for <'de> Deserialize<'de> + Debug>(knockoff_factories: &str) -> Option<T> {
        File::open(knockoff_factories)
            .as_mut()
            .map(|f| {
                info!("Opened knockoff factories file.");
                let mut all_value = "".to_string();
                f.read_to_string(&mut all_value)
                    .expect("Could not read factories.toml");
                all_value
            })
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e.to_string()))
            // .map_err(err::log_err("Failed to parse: "))
            .map(|all_value| {
                toml::from_str::<T>(all_value.as_str())
                    // .map_err(err::log_err("Failed to parse: "))
                    .map(|s| {
                        info!("Parsed factory stages: {:?}", s);
                        s
                    })
                    .ok()
            })
            .ok()
            .flatten()
    }

    pub fn get_deps_and_providers(factories: &mut Factories) -> (&mut Option<Value>, HashMap<String, Provider>) {
        let providers = factories.get_providers().clone();
        let deps = factories.get_factories();
        (deps, providers)
    }

    fn write_error_creating_out_lib(out_lib_rs: &String) -> String {
        format!("Tried to create {}. Error creating knockoff providers gen: .", out_lib_rs)
    }

    pub fn default_stage_deps(phase: &Phase, stage: &str) -> Value {
        info!("Adding stage deps for {}, {}", &phase.to_string(), stage);
        let phase_deps_path = project_directory_path!()
            .join("codegen_resources")
            .join("default_stage_deps.toml")
            .to_path_buf();

        let default_gen = codegen_utils::io_utils::read_dir_to_file(&phase_deps_path)
            .flat_map_res(|read| toml::from_str(&read)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string())))
            .or::<Table>(Ok(Value::Table(Table::new())))
            .unwrap();

        let mut first = get_key(vec![SearchType::FieldKey(Phase::All.to_string()), SearchType::FieldKey(Phase::All.to_string())], &default_gen)
            .or(Ok::<Value, SearchTypeError>(Value::Table(Table::new()))).unwrap();

        get_key(vec![SearchType::FieldKey(phase.to_string()), SearchType::FieldKey(stage.to_string())], &default_gen)
            .ok()
            .map(|all| Self::add_all_to_table(&mut first, all));

        first
    }

    fn add_all_to_table(mut first: &mut Value, all: Value) {
        let _ = all.as_table().iter()
            .flat_map(|all| all.iter())
            .for_each(|(k, v)| {
                first.as_table_mut()
                    .flat_map_opt(|to_add| {
                        if !to_add.contains_key(k) {
                            to_add.insert(k.to_string(), v.clone());
                        }

                        None::<bool>
                    });
            });
    }

    pub fn default_phase_deps(phase: &Phase) -> Value {
        info!("Adding phase deps for {}", &phase.to_string());
        let phase_deps_path = project_directory_path!()
            .join("codegen_resources")
            .join("default_phase_deps.toml")
            .to_path_buf();

        let default_gen = codegen_utils::io_utils::read_dir_to_file(&phase_deps_path)
            .flat_map_res(|read| toml::from_str(&read)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string())))
            .or::<Table>(Ok(Value::Table(Table::new())))
            .unwrap();

        let mut first = get_key(vec![SearchType::FieldKey(Phase::All.to_string())], &default_gen)
            .or(Ok::<Value, SearchTypeError>(Value::Table(Table::new()))).unwrap();

        get_key(vec![SearchType::FieldKey(phase.to_string())], &default_gen)
            .ok()
            .map(|all| Self::add_all_to_table(&mut first, all));

        info!("{:?} are deps after for {:?}", first.to_string(), phase.to_string());
        first
    }

    pub(crate) fn add_delegating_deps(stages: Vec<&str>, phase: &Phase, deps: &mut Table) {
        info!("Adding delegating deps. for {:?}, {:?} and {:?}", phase.prefix(), deps.to_string(), stages);
        for stage in stages {
            let mut table = Table::new();
            let factory_name = format!("{}{}", phase.prefix(), stage);
            let factory_path = format!("../{}", &factory_name);
            info!("Adding {}", &factory_name);
            table.insert("path".to_string(), Value::String(factory_path));
            table.insert("name".to_string(), Value::String(factory_name.clone()));
            table.insert("registry".to_string(), Value::String("estuary".to_string()));
            deps.insert(factory_name.to_string(), Value::Table(table.clone()));
        }

        // Self::remove_paths_from_dependencies_table(&mut dep_table, base_dir);
    }

    pub fn write_delegating_delegators(
        factories_parser: &mut FactoryStages,
        version: &String,
        phase: &Phase
    ) -> String {
        let stages = Self::retrieve_stages(factories_parser);
        let providers = Factories::create_providers_for_stages(stages.clone(), version, phase);
        providers.to_string()
    }

    fn retrieve_stages(factories_parser: &mut FactoryStages) -> Vec<String> {
        factories_parser.stages.iter()
            .map(|(stage, e)| stage.clone())
            .collect()
    }

    fn write_lib_rs(factory: &mut Factories) -> String {
        info!("Here are factories: {:?}", factory);
        let parsed_factories = FactoriesParser::tokens(factory);
        info!("Here are parsed: {:?}", parsed_factories.to_string().as_str());
        parsed_factories.to_string()
    }

}