use std::{env, fs};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fs::File;
use collection_util::MultiMap;
use collection_util::IntoMultiMap;
use std::path::Path;
use toml::{Table, Value};
use proc_macro2::{Ident, Span};
use syn::parse_str;
use std::io::{Error, Read, Write};
use std::marker::PhantomData;
use crate::parse_container_modifier::ParseContainerModifierProvider;
use crate::provider::{DelegatingProvider};
use crate::parse_provider::ParseProvider;
use crate::token_provider::TokenProvider;
use executors::common::Executor;
use crate::framework_token_provider::FrameworkTokenProvider;
use serde::{Deserialize, Serialize};
use toml::map::Map;
use crate::item_modifier::ItemModifierProvider;

use crate::provider::ProviderProvider;
use crate::profile_tree_modifier::ProfileTreeModifierProvider;
use crate::profile_tree_finalizer::ProfileTreeFinalizerProvider;


use knockoff_logging::*;
use std::sync::Mutex;
use crate::logger_lazy;

import_logger!("factories_parser.rs");

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
    Providers
}

impl Phase {
    pub fn prefix(&self) -> &'static str {
        match self {
            Phase::PreCompile => "knockoff_precompile_gen",
            Phase::Providers => "knockoff_providers_gen"
        }
    }
}

macro_rules! factories {


    ($(($ty:ident, $factory_name:ident, $factory_name_lit:literal, $delegator_name_lit:literal)),*) => {

        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub struct Factory {
            pub values: Option<HashMap<String, Provider>>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub struct Factories {
            $(
                pub $factory_name: Option<Factory>,
            )*
            pub dependencies: Option<Value>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub struct FactoryStages {
            pub stages: HashMap<String, Factories>,
            pub gen_deps: Option<Value>,
        }

        #[derive(Serialize, Deserialize, Clone, Debug)]
        pub struct FactoryPhases {
            pub phase_deps: Option<Value>,
            pub phases: HashMap<Phase, FactoryStages>,
        }


        impl Factories {

            pub fn create_import_names(stages: &Vec<String>) -> HashMap<String, HashMap<String, String>> {
                let mut out_map = HashMap::new();
                stages.iter().for_each(|stage| {
                    let mut next_stage = HashMap::new();
                    $(
                        let import_name = format!("{}{}", $delegator_name_lit, stage);
                        next_stage.insert($factory_name_lit.to_string(), import_name.to_string());
                    )*
                    out_map.insert(stage.to_string(), next_stage);
                });
                out_map
            }

            pub fn create_providers_for_stages(
                stages: Vec<String>, version: &String, phase: &Phase
            ) -> TokenStream {
                let mut out: HashMap<String, Vec<Provider>> = HashMap::new();
                let import_names = Self::create_import_names(&stages);
                    stages.iter().for_each(|stage| {
                        let stage_map = import_names.get(stage).unwrap();
                        $(
                            let mut table = Table::new();
                            table.insert("path".to_string(), Value::String(format!("../{}{}", phase.prefix(), stage).to_string()));
                            table.insert("version".to_string(), Value::String(version.to_string().clone()));
                            table.insert("registry".to_string(), Value::String("estuary".to_string()));
                            let mut dep_data = Value::Table(table);
                            let import_name = format!("{}{}", $delegator_name_lit, stage);
                            let import_name = format!("{}{}", $delegator_name_lit, stage);
                            let use_statement = format!(
                                "use {}{}::{} as {};",
                                phase.prefix(), stage, $delegator_name_lit,
                                import_name)
                            .to_string();
                            let provider = Provider {
                                 provider_data: Some(ProviderData {
                                    provider_path: stage_map.get($factory_name_lit).cloned(),
                                    provider_ident: Some(format!("{}{}Ident", $factory_name_lit, stage)),
                                    provider_path_use_statement: Some(use_statement)
                                 }),
                                 dependency_data: Some(dep_data)
                            };
                            collection_util::add_to_multi_value(&mut out, provider, $factory_name_lit.to_string());
                        )*
                    });
                let mut out_ts = TokenStream::default();
                $(
                    out.get($factory_name_lit).map(|providers| {
                        let providers = providers.iter().map(|p| p).collect();
                        let tokens = $ty::get_tokens(&providers);
                        out_ts.append_all(tokens);
                    });
                )*
                out_ts
            }


            pub fn create_use_statements(stages: Vec<String>, phase: &Phase) -> Vec<String> {
                stages.iter().flat_map(|s| {
                    let mut use_statements = vec![];
                    $(
                        let import_name = format!("{}{}", $delegator_name_lit, s);
                        use_statements.push(format!("use {}{}::{} as {}",
                        phase.prefix(), s, $delegator_name_lit, import_name).to_string());
                    )*
                    use_statements
                }).collect::<Vec<String>>()
            }

            pub fn has_factories(&self) -> Vec<&'static str> {
                let mut has: Vec<&'static str> = vec![];
                $(
                    if self.$factory_name.is_some() {
                        has.push($factory_name_lit);
                    }
                )*
                has
            }

            pub fn missing_factories(&self) -> Vec<&'static str> {
                let mut has: Vec<&'static str> = vec![];
                $(
                    if self.$factory_name.is_none() {
                        has.push($factory_name_lit);
                    }
                )*
                has
            }
        }

        impl FactoryStages {
            pub fn all_factory_names() -> Vec<&'static str> {
                let mut out: Vec<&'static str> = vec![];
                $(
                    out.push($factory_name_lit);
                )*
                out
            }
        }
    }

}

macro_rules! providers {


    ($(($ty:ident, $factory_name:ident, $factory_name_lit:literal)),*) => {

        use proc_macro2::TokenStream;
        use quote::TokenStreamExt;

        impl DelegatingProvider<Factories> for FactoriesParser {
            fn tokens(t: &Factories) -> TokenStream {
                let mut ts = TokenStream::default();
                $(
                    ts.append_all($ty::tokens(t));
                )*
                ts
            }
        }

        impl FactoriesParser {
            pub fn get_default_tokens_for(name: &str) -> TokenStream {
                $(
                    if name == $factory_name_lit {
                        return $ty::get_tokens(&vec![]);
                    }
                )*
                TokenStream::default()
            }
        }

        $(
            impl DelegatingProvider<Factories> for $ty {
                fn tokens(t: &Factories) -> TokenStream {
                    if t.$factory_name.as_ref().is_none() {
                        $ty::get_tokens(&vec![])
                    } else {
                        let t = t.$factory_name.as_ref().unwrap();
                        let providers = &t
                            .values
                            .iter()
                            .flat_map(|val| val
                                .values()
                                .collect::<Vec<&Provider>>()
                            )
                            .collect();
                        let ts = $ty::get_tokens(providers);
                        ts
                    }
                }
            }
        )*

        impl Factories {
            pub fn get_providers(&self) -> HashMap<String, Provider> {
                let mut provider_map = HashMap::new();
                $(
                    self.insert_provider(&mut provider_map, &self.$factory_name);
                )*
                provider_map
            }
            pub fn get_factories(&self) -> Option<Value> {
                self.dependencies.clone()
            }
        }

    }
}

factories!(
    (ParseProvider, parse_provider, "parse_provider", "DelegatingParseProvider"),
    (TokenProvider, token_provider, "token_provider", "DelegatingTokenProvider"),
    (FrameworkTokenProvider, framework_token_provider, "framework_token_provider", "DelegatingFrameworkTokenProvider"),
    (ParseContainerModifierProvider, parse_container_modifier, "parse_container_modifier", "DelegatingParseContainerModifierProvider"),
    (ProfileTreeModifierProvider, profile_tree_modifier_provider, "profile_tree_modifier_provider", "DelegatingProfileTreeModifierProvider"),
    (ProfileTreeFinalizerProvider, profile_tree_finalizer, "profile_tree_finalizer", "DelegatingProfileTreeFinalizerProvider"),
    (ItemModifierProvider, item_provider, "item_provider", "DelegatingItemModifier")
);

providers!(
    (ParseProvider, parse_provider, "parse_provider"),
    (TokenProvider, token_provider, "token_provider"),
    (FrameworkTokenProvider, framework_token_provider, "framework_token_provider"),
    (ParseContainerModifierProvider, parse_container_modifier, "parse_container_modifier"),
    (ProfileTreeModifierProvider, profile_tree_modifier_provider, "profile_tree_modifier_provider"),
    (ProfileTreeFinalizerProvider, profile_tree_finalizer, "profile_tree_finalizer"),
    (ItemModifierProvider, item_provider, "item_provider")
);


impl Factories {

    fn insert_provider(&self, mut provider_map: &mut HashMap<String, Provider>,
                       option: &Option<Factory>) {
        option.as_ref()
            .iter()
            .flat_map(|token_provider| {
                token_provider.values.iter()
            })
            .for_each(|token_provider | {
                token_provider.iter().for_each(|t| {
                    provider_map.insert(t.0.clone(), t.1.clone());
                });
            });
    }

    fn insert_dependency(&self, name: &str, mut dep_map: &mut HashMap<String, Option<Value>>, dep: &Option<Value>
    ) {
        dep_map.insert(name.to_string(), dep.clone());
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

    pub fn get_starting_toml_prelude(stage_id: &String, version: &String, phase: &Phase) -> String {
        let mut prelude =
            format!("[package]
name = \"{}{}\"
version = \"{}\"
edition = \"2021\"
", phase.prefix(), stage_id, &version);
        prelude.to_string()
    }

    pub fn parse_factories_value<T: for <'de> Deserialize<'de> + Debug>(
        knockoff_factories: &str
    ) -> Option<T> {

        File::open(knockoff_factories)
            .as_mut()
            .map(|f| {
                info!("Opened knockoff factories file.");
                let mut all_value = "".to_string();
                f.read_to_string(&mut all_value)
                    .expect("Could not read factories.toml");
                all_value
            })
            .map(|all_value| {
                toml::from_str::<T>(all_value.as_str())
                    .map_err(|err| {
                        error!("{}", err.to_string());
                    })
                    .map(|s| {
                        info!("Parsed factory stages: {:?}", s);
                        s
                    })
                    .ok()
            })
            .ok()
            .flatten()
    }

    pub fn get_deps_and_providers(factories: &Factories) -> (Option<Value>, HashMap<String, Provider>) {
        let deps = factories.get_factories();
        let providers = factories.get_providers();
        (deps, providers)
    }

    fn write_error_creating_out_lib(out_lib_rs: &String, err: &dyn std::error::Error)  {
        error!("Tried to create {}. Error creating knockoff providers gen: {}.", out_lib_rs, err.to_string().as_str());
    }

    fn get_create_directories<'a>(
        factory_stages: &'a FactoryStages,
        base_dir: &'a String,
        out_directory: &'a String,
        phase: &'a Phase
    ) -> Vec<(String, String, String, &'a String, Option<&'a Factories>, &'a String)> {

        log_message!("{} is out", &out_directory);
        let (out_lib_dir, cargo_toml, out_lib_rs) = Self::create_dirs(&out_directory, &String::default(), phase);
        let mut out_dirs = vec![];
        out_dirs.push((out_lib_dir, cargo_toml, out_lib_rs, base_dir, None, base_dir));

        for (stage_id, factories) in factory_stages.stages.iter() {
            let (out_lib_dir, cargo_toml, out_lib_rs) = Self::create_dirs(&out_directory, stage_id, phase);
            out_dirs.push((out_lib_dir, cargo_toml, out_lib_rs, base_dir, Some(factories), stage_id));
        }

        out_dirs

    }

    fn create_dirs(out_directory: &String, stage_id: &String, phase: &Phase) -> (String, String, String) {
        let (out_lib_dir, cargo_toml, out_lib_rs) = Self::get_lib_build_dirs(out_directory, stage_id, phase);
        let _ = fs::remove_dir_all(&out_lib_dir)
            .map_err(|err| {
                Self::write_error_creating_out_lib(&out_lib_rs, &err);
                Ok::<(), Error>(())
            });
        let _ = fs::create_dir_all(&out_lib_dir)
            .map_err(|err| {
                Self::write_error_creating_out_lib(&out_lib_rs, &err);
                Ok::<(), Error>(())
            });
        (out_lib_dir, cargo_toml, out_lib_rs)
    }

    fn get_lib_build_dirs(out_directory: &String, stage_id: &String, phase: &Phase) -> (String, String, String) {
        let mut out_lib_dir = out_directory.clone();
        out_lib_dir += format!("/{}{}/src", phase.prefix(), stage_id).as_str();
        let mut cargo_toml = out_directory.clone();
        cargo_toml += format!("/{}{}/Cargo.toml", phase.prefix(), stage_id).as_str();
        let mut out_lib_rs = out_directory.clone();
        out_lib_rs += format!("/{}{}/src/lib.rs", phase.prefix(), stage_id).as_str();
        (out_lib_dir, cargo_toml, out_lib_rs)
    }

    pub fn write_phase(
        version: &String, knockoff_factories: &String,
        base_dir: &String, out_directory: &String, phase: &Phase
    ) -> Option<FactoryStages> {
        info!("Writing {}.", phase.prefix());
        Self::parse_factories_value::<FactoryPhases>(knockoff_factories)
            .as_mut()
            .map(|f| {
                let f = f.phases.remove(phase)
                    .or_else(|| {
                        panic!("Phase was not contained in knockoff gens.");
                    })
                    .unwrap();
                let stages = f.stages.keys().map(|s| s.as_str()).collect::<Vec<&str>>();
                info!("Parsed factories value: {:?}.", &f);
                let mut directories_created = Self::get_create_directories(&f, &base_dir, out_directory, phase);
                info!("Found {} directories to be created: {:?}.", directories_created.len(), directories_created);
                if directories_created.len() > 1 {
                    for (out_lib_dir, cargo_toml, out_lib_rs, base_dir, factories, stage) in directories_created[1..].iter() {
                        Self::write_cargo_toml(cargo_toml, base_dir, stage, version, factories.as_ref().unwrap(), &f, phase);
                    }
                }
                if directories_created.len() != 0 {
                    let (out_lib_dir, cargo_toml, out_lib_rs, base_dir, factories, stage)
                        = directories_created.remove(0);
                    Self::write_delegating_cargo_toml(&cargo_toml, base_dir, version, stages, &f, phase);
                }
                f
            })
    }

    pub fn write_delegating_cargo_toml(
        cargo_file: &str,
        base_dir: &String,
        version: &String,
        stages: Vec<&str>,
        factory_stages: &FactoryStages,
        phase: &Phase
    ) {

        info!("Writing output directory: {:?} {:?}", &base_dir, cargo_file);

        let path = Path::new(cargo_file);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
        log_message!("Opening {}", &cargo_file);
        let mut cargo_file = File::create(path).unwrap();
        let cargo_str = Self::get_delegating_cargo_toml(&base_dir, version, stages, factory_stages, phase);
        cargo_file.write_all(cargo_str.as_bytes())
            .unwrap();
    }

    pub fn write_cargo_toml(
        cargo_file: &str,
        base_dir: &String,
        stage: &String,
        version: &String,
        factories: &Factories,
        factories_parser: &FactoryStages,
        phase: &Phase
    ) {

        info!("Writing output directory: {:?} {:?}", &base_dir, cargo_file);

        let path = Path::new(cargo_file);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
        log_message!("Opening {}", &cargo_file);
        let mut cargo_file = File::create(path).unwrap();
        let cargo_str = Self::get_cargo_toml_string(
            &base_dir, factories,
            &stage, version, factories_parser,
            phase
        );
        cargo_file.write_all(cargo_str.as_bytes())
            .unwrap();
    }

    pub(crate) fn get_delegating_cargo_toml(
        base_dir: &String, version: &String, stages: Vec<&str>,
        factory_stages: &FactoryStages, phase: &Phase
    ) -> String {
        use std::fmt::Write;
        let mut cargo_file = "".to_string();
        writeln!(&mut cargo_file, "{}", Self::get_starting_toml_prelude(&String::default(), version, phase).as_str()).unwrap();
        let mut deps_table = Table::new();
        let needs = Self::get_needs_gen(factory_stages);
        let deps = factory_stages.gen_deps.as_ref();

        if needs.len() != 0 {
            assert!(deps.is_some());
            let table = deps.as_ref().unwrap().as_table().unwrap();
            for (k, v) in table.iter() {
                deps_table.insert(k.clone(), v.clone());
            }
        }

        let mut dep_table = Table::new();
        dep_table.insert("dependencies".to_string(), Value::Table(deps_table));
        // Self::remove_paths_from_dependencies_table(&mut dep_table, base_dir);

        stages.iter()
            .map(|mut dep| {
                let mut next_dep =  Table::new();
                next_dep.insert("name".to_string(), Value::String(format!("{}{}", phase.prefix(), dep)));
                next_dep.insert("path".to_string(), Value::String(format!("../{}{}", phase.prefix(), dep)));
                (format!("{}{}", phase.prefix(), dep.to_string()), Value::Table(next_dep))
            })
            .for_each(|(name, table)| {
                dep_table.get_mut("dependencies")
                    .map(|d| d.as_table_mut()
                        .map(|t| t.insert(name, table))
                    );
            });

        writeln!(&mut cargo_file, "{}", dep_table.to_string()).unwrap();
        writeln!(&mut cargo_file, "[workspace]")
            .unwrap();
        cargo_file
    }


    pub(crate) fn get_cargo_toml_string(
        base_dir: &String, factories: &Factories,
        stage_id: &String, version: &String,
        factories_parser: &FactoryStages,
        phase: &Phase
    ) -> String {

        use std::fmt::Write;
        info!("Writing cargo toml for {:?}", factories);
        let (mut knockoff_providers_dep, parsed_factories)
            = FactoriesParser::get_deps_and_providers(factories);
        let mut cargo_file = "".to_string();
        writeln!(&mut cargo_file, "{}", Self::get_starting_toml_prelude(stage_id, version, phase).as_str()).unwrap();
        info!("Found deps: {:?} for phase {}.", knockoff_providers_dep, phase.prefix());
        if knockoff_providers_dep.as_ref().is_some() {
            let mut dep = knockoff_providers_dep.unwrap().clone();
            info!("Adding dependency refs for providers: {:?}", parsed_factories);
            for (factory_name, provider) in parsed_factories.iter() {

                if provider.dependency_data.as_ref().is_some() {
                    dep.as_table_mut().unwrap().insert(
                        factory_name.into(),
                        provider.dependency_data.clone().unwrap().clone());
                }
            }
            info!("Found dep: {:?}", dep);
            let mut dep_table = Table::new();
            dep_table.insert("dependencies".to_string(), dep);
            log_message!("Removing paths from {} Cargo.toml because not knockoff dev.", phase.prefix());
            // Self::remove_paths_from_dependencies_table(&mut dep_table, base_dir);
            let dep_table_str = dep_table.to_string();
            info!("Writing file {:?}", &dep_table_str);
            writeln!(&mut cargo_file, "{}", &dep_table_str).unwrap();
            writeln!(&mut cargo_file, "[workspace]")
                .unwrap();
        }
        cargo_file
    }

    fn remove_paths_from_dependencies_table(mut out_table: &mut Map<String, Value>, base_dir: &String) {
        // if the module_macro_lib library is in the project directory, then keep the ../path in the
        // Cargo.toml.
        info!("Removing from {:?}", &out_table);
        out_table.get_mut("dependencies")
            .map(|out| out.as_table_mut()
                .map(|t| {
                    info!("Removing from dependency table {:?}", &t);
                    let keys = t.keys()
                        .map(|s| s.clone())
                        .collect::<Vec<String>>();
                    keys.iter().for_each(|key| {
                        let mut table_mut = t.get_mut(key).unwrap().as_table_mut();
                        info!("Removing from dependency {:?}", &table_mut);
                        if table_mut.as_ref().is_none() {
                            return;
                        }
                        if table_mut.as_ref().is_some() {
                            let mut path = table_mut.as_mut().unwrap().get("path");
                            if path.as_ref().is_none() {
                                info!("Could not remove path, as it did not exist");
                                return;
                            }
                            info!("Going to remove path: {:?} from deps.", &path);
                            let path_buf = Path::new(base_dir).join(Path::new(&key));
                            let target_path_buf = Path::new(base_dir).join("target").join(key);
                            if !path_buf.exists() && !target_path_buf.exists() {
                                info!("Removing path for {}, as it does not point to a local path", path_buf.to_str().unwrap());
                                table_mut.as_mut().unwrap().remove("path");
                                info!("After removal: {:?}.", &table_mut);
                            }
                        }
                    });
                })
            );
    }

    pub fn write_delegating_delegators(
        out_file: &mut File,
        factories_parser: &mut FactoryStages,
        version: &String,
        phase: &Phase
    ) {
        let stages = Self::retrieve_stages(factories_parser);
        let providers = Factories::create_providers_for_stages(stages.clone(), version, phase);

        info!("Created providers: {:?}", providers.to_string().as_str());
        let _ = write!(out_file, "{}", providers.to_string())
            .map_err(|e| {
                error!("Error writing gen {}: {:?}", phase.prefix(), e);
            });

    }

    fn get_needs_gen(factories_parser: &FactoryStages) -> Vec<&str> {
        let existing = factories_parser.stages.values()
            .flat_map(|s| s.has_factories())
            .collect::<Vec<&'static str>>();

        let all_factory_names = FactoryStages::all_factory_names();
        let mut needs_factory_default = vec![];
        for a in all_factory_names.into_iter() {
            if !existing.contains(&a) {
                needs_factory_default.push(a.clone());
            }
        }
        needs_factory_default
    }


    pub fn write_tokens_lib_rs(
        mut factories_parser: FactoryStages,
        out_directory: &String,
        version: &String,
        phase: &Phase
    ) {

        let (_out_lib_dir, _cargo_toml, out_lib_rs) = Self::get_lib_build_dirs(&out_directory, &String::default(), phase);

        let _ = File::create(out_lib_rs)
            .as_mut()
            .map(|f| Self::write_delegating_delegators(f, &mut factories_parser, version, phase))
            .or_else(|e| {
                error!("Error writing pub use {:?}", e);
                Err(e)
            });


        for (stage, factory) in factories_parser.stages.into_iter() {
            let (out_lib_dir, cargo_toml, out_lib_rs)
                = Self::get_lib_build_dirs(&out_directory, &stage, phase);
            let lib_rs_file_path = Path::new(&out_lib_rs);
            if stage == String::default() {
                continue;
            } else {
                let _ = fs::remove_file(lib_rs_file_path).map_err(|e| {
                    error!("Error removing {:?}: {:?}", lib_rs_file_path, &e);
                });

                File::create(lib_rs_file_path)
                    .map(|mut lib_rs_file| Self::write_lib_rs(&mut lib_rs_file, factory))
                    .ok()
                    .flatten()
                    .or_else(|| {
                        log_message!("Could not write to lib.rs file.");
                        None
                    });
            }
        }

    }

    fn retrieve_stages(factories_parser: &mut FactoryStages) -> Vec<String> {
        factories_parser.stages.iter()
            .map(|(stage, e)| stage.clone())
            .collect()
    }

    fn retrieve_providers_by_stage(factories_parser: &mut FactoryStages) -> HashMap<String, Vec<HashMap<String, Provider>>> {
        let stages = factories_parser.stages.iter()
            .clone().map(|e| (e.0.clone(), e.1.get_providers()))
            .collect_multi();
        stages.0
    }

    fn write_lib_rs(mut lib_rs_file: &mut File, factory: Factories) -> Option<()> {
        let parsed_factories = FactoriesParser::tokens(&factory);
        writeln!(&mut lib_rs_file, "{}", parsed_factories.to_string().as_str())
            .ok()
    }


}