use std::collections::HashMap;
use std::env;
use std::path::Path;
use serde::{Deserialize, Serialize};
use syn::Item;
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::{BuildParseContainer, ItemModifier, ModuleParser, parse_module_into_container, ParseContainer, ProfileTree, ProfileTreeBuilder, ProfileTreeModifier, ProfileTreeTokenProvider};
use toml;
use codegen_utils::parse::read_file_to_str;
use knockoff_env::get_project_dir;


use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use proc_macro2::TokenStream;
use codegen_utils::project_directory;
import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/module_precompile.log"));

use knockoff_precompile_gen::{DelegatingFrameworkTokenProvider, DelegatingParseContainerModifierProvider, DelegatingItemModifier, DelegatingProfileTreeModifierProvider, DelegatingProfileTreeFinalizerProvider, DelegatingTokenProvider, DelegatingParseProvider};


#[derive(Deserialize)]
pub struct PrecompileFactories {
    factories: HashMap<String, PrecompileMetadata>
}

#[derive(Deserialize)]
pub struct PrecompileMetadata {
    processor_path: String,
    processor_files: Vec<String>,
    processor_ident: String
}

pub fn get_tokens(processor_name: &str) -> TokenStream {
    info!("Starting precompile parsing for {}", processor_name);
    let factories = knockoff_factories();
    let precompiled = precompiled_metadata(
        factories, processor_name);
    if precompiled.is_some() {
        let ts: Vec<TokenStream> = parse_precompile_inputs(&precompiled)
            .into_iter()
            .flat_map(|f| f.items.into_iter())
            .flat_map(|mut item_parsed| to_parse_container(&precompiled, item_parsed))
            .map(|mut parse_container| build_profile_tree(&mut parse_container))
            .map(|mut profile_tree| DelegatingTokenProvider::new(&mut profile_tree))
            .map(|token_provider| token_provider.generate_token_stream())
            .collect();
        let mut out_tokens = TokenStream::default();
        out_tokens.extend(ts);
        out_tokens
    } else {
        TokenStream::default()
    }
}

fn build_profile_tree(parse_container: &mut ParseContainer) -> ProfileTree {
    let profile_tree_builder = vec![
        Box::new(DelegatingProfileTreeModifierProvider::new(
            &parse_container.injectable_types_builder)
        ) as Box<dyn ProfileTreeModifier>
    ];
    ProfileTreeBuilder::build_profile_tree(
        &mut parse_container.injectable_types_builder,
        profile_tree_builder,
        &mut parse_container.provided_items
    )
}

pub struct ParseContainerBuilder;

impl BuildParseContainer for ParseContainerBuilder {
    fn build_parse_container(&self, parse_container: &mut ParseContainer) {
        info!("Called build parse container in precompile with parse container: {:?}", parse_container);
    }
}

fn to_parse_container(precompiled: &Option<PrecompileMetadata>, item_parsed: Item) -> Vec<ParseContainer> {
    match item_parsed {
        Item::Mod(item_mod) => {
            info!("Testing if {:?} contains processor ident {:?}.",
                SynHelper::get_str(&item_mod.ident),
                precompiled.as_ref().unwrap().processor_ident.as_str());
            let attr = SynHelper::get_attr_from_vec(
                &item_mod.attrs,
                &vec![precompiled.as_ref().unwrap().processor_ident.as_str()]
            );
            if attr.is_some() {
                let module_ident = SynHelper::get_str(&item_mod.ident);
                info!("Found mod {:?} with processor {:?}. Parsing it into ParseContainer.",
                    &module_ident, &attr);
                let mut module_parser = ModuleParser {
                    delegating_parse_container_updater: DelegatingParseProvider {},
                    delegating_parse_container_modifier: DelegatingParseContainerModifierProvider::new(),
                    delegating_parse_container_builder: ParseContainerBuilder {},
                    delegating_parse_container_item_modifier: DelegatingItemModifier::new(),
                    delegating_parse_container_finalizer: DelegatingProfileTreeFinalizerProvider {},
                };
                let parse_containers = parse_module_into_container(
                        &mut Item::Mod(item_mod),
                        &mut module_parser
                    )
                    .into_iter()
                    .map(|p| {
                        info!("Parsed module {:?} into parse container {:?} in precompile.",
                        &module_ident, p);
                        p
                    })
                    .collect::<Vec<_>>();
                info!("Parsed {} number of parse containers in precompile.", parse_containers.len());
                parse_containers
            } else {
                info!("Found item {:?}, but did not contain processor ", SynHelper::get_str(&item_mod.ident));
                item_mod.content.iter().flat_map(|c| c.1.clone())
                    .flat_map(|c| to_parse_container(precompiled, c))
                    .collect()
            }
        }
        _ => vec![]
    }
}

fn parse_precompile_inputs(precompiled_metadata: &Option<PrecompileMetadata>) -> Vec<syn::File> {
    precompiled_metadata
        .iter()
        .flat_map(|read_value| {
            read_value.processor_files.iter()
                .flat_map(|processor_file| SynHelper::open_syn_file_from_str_path_name(processor_file)
                    .or_else(|| {
                        error!("Could not read processor file: {:?}", processor_file);
                        None
                    })
                    .into_iter()
                )
                .into_iter()
        })
        .collect()
}

fn knockoff_factories() -> String {
    let knockoff_factories = env::var("KNOCKOFF_PRECOMPILE")
        .ok()
        .or(Some(get_project_dir("codegen_resources/knockoff_precompile.toml")))
        .unwrap();
    info!("Loading factories from {:?}", &knockoff_factories);
    knockoff_factories
}

fn precompiled_metadata(knockoff_factories: String, precompile_factory: &str) -> Option<PrecompileMetadata>{
    read_file_to_str(&Path::new(&knockoff_factories).to_path_buf())
        .map_err(|e| {
            error!("Error reading file {:?}", e);
            e
        })
        .ok()
        .map(|read_value| {
            toml::from_str::<PrecompileFactories>(read_value.as_str())
                .map_err(|e| {
                    error!("Error serializing to toml: {:?}", e);
                    e
                })
                .ok()
                .as_mut()
                .map(|f| f.factories.remove(precompile_factory))
        })
        .flatten()
        .flatten()
}

