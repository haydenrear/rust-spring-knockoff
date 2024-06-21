use std::collections::HashMap;
use std::fmt::Debug;

use std::sync::Mutex;
use crate::logger_lazy;
use proc_macro2::TokenStream;
use serde::{Deserialize, Serialize};
use toml::{Table, Value};

use knockoff_logging::*;
use knockoff_logging::info;

use crate::factories_parser::{Provider, Phase, ProviderData};
use crate::framework_token_provider::FrameworkTokenProvider;
use crate::item_modifier::ItemModifierProvider;
use crate::mutable_module_modifier_provider::MutableMacroModifierProvider;
use crate::parse_container_modifier::ParseContainerModifierProvider;
use crate::parse_provider::ParseProvider;
use crate::profile_tree_finalizer::ProfileTreeFinalizerProvider;
use crate::profile_tree_modifier::ProfileTreeModifierProvider;
use crate::token_provider::TokenProvider;
use crate::provider::ProviderProvider;

import_logger!("factories_parser.rs");

#[macro_export]
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

            pub fn create_providers_for_stages(stages: Vec<String>, version: &String, phase: &Phase) -> TokenStream {
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
                    if !out.contains_key($factory_name_lit) {
                        info!("Did not contain key for {}.", $factory_name_lit);
                    } else {
                        out.get($factory_name_lit)
                            .map(|providers| {
                                info!("Added key for {} and providers {:?}.", $factory_name_lit, &providers);
                                let providers = providers.iter().map(|p| p).collect();
                                let tokens = $ty::get_tokens(&providers);
                                out_ts.extend(tokens);
                            });
                    }
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

factories!(
    (ParseProvider, parse_provider, "parse_provider", "DelegatingParseProvider"),
    (TokenProvider, token_provider, "token_provider", "DelegatingTokenProvider"),
    (FrameworkTokenProvider, framework_token_provider, "framework_token_provider", "DelegatingFrameworkTokenProvider"),
    (ParseContainerModifierProvider, parse_container_modifier, "parse_container_modifier", "DelegatingParseContainerModifierProvider"),
    (ProfileTreeModifierProvider, profile_tree_modifier_provider, "profile_tree_modifier_provider", "DelegatingProfileTreeModifierProvider"),
    (ProfileTreeFinalizerProvider, profile_tree_finalizer, "profile_tree_finalizer", "DelegatingProfileTreeFinalizerProvider"),
    (ItemModifierProvider, item_modifier, "item_modifier", "DelegatingItemModifier"),
    (MutableMacroModifierProvider, mutable_macro_modifier_provider, "mutable_macro_modifier_provider", "DelegatingMutableMacroModifierProvider")
);

