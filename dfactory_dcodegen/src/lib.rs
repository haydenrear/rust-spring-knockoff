use std::env;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use factories_codegen::factories_parser::{FactoriesParser, Phase};
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Item, ItemImpl, ItemMod, Visibility};
use toml::Table;
use codegen_utils::{FlatMapOptional, program_src, project_directory, project_directory_path, user_program_src};
use codegen_utils::{get_build_project_dir, get_project_base_build_dir, get_project_dir};
use codegen_utils::syn_helper::SynHelper;
use crate_gen::CrateWriter;

import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/precompile_codegen.log"));

use knockoff_dfactory_gen::{DelegatingFrameworkTokenProvider, DelegatingParseContainerModifierProvider, DelegatingItemModifier,
                            DelegatingProfileTreeFinalizerProvider, DelegatingTokenProvider, DelegatingParseProvider};

use module_macro_shared::{BuildParseContainer, ModuleParser, parse_module_into_container, ParseContainer, ProfileProfileTreeModifier, ProfileTreeBuilder, ProfileTreeModifier, ItemModifier, ProfileTreeFrameworkTokenProvider, ProfileTreeTokenProvider, do_parse_container, do_container_modifications, do_modify, ItemParser};
use module_macro_shared::item_mod_parser::ItemModParser;
use optional::FlatMapResult;
use module_macro_shared::ParseContainerItemUpdater;

pub struct ParseContainerBuilder;

impl BuildParseContainer for ParseContainerBuilder {
    fn build_parse_container(&self, parse_container: &mut ParseContainer) {
        info!("Called build parse container in precompile with parse container: {:?}", parse_container);
    }
}

/// This will import the code from all dfactory that iterates over the program and generate code to be imported into
///   the precompile macro, and then iterate over the program with and return generated token stream, to be saved as
///   knockoff_dfactory.
pub fn write_d_factory_crate() -> Option<String> {
    /// To generate here, import the DFactory crate, then generate a crate from the tokens generated
    /// from that crate.
    let mut module_parser = ModuleParser {
        delegating_parse_container_updater: DelegatingParseProvider {},
        delegating_parse_container_modifier: DelegatingParseContainerModifierProvider::new(),
        delegating_parse_container_builder: ParseContainerBuilder {},
        delegating_parse_container_item_modifier: DelegatingItemModifier::new(),
        delegating_parse_container_finalizer: DelegatingProfileTreeFinalizerProvider {},
    };

    codegen_utils::io_utils::open_file_read(&user_program_src!().join("lib.rs"))
        .map_err(err::log_err("Failed to open lib file: "))
        .flat_map_res(|mut f| SynHelper::parse_syn_file_to_res(&mut f)
            .map_err(err::log_err("Failed to parse syn file."))
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e.to_string()))
        )
        .ok()
        .flat_map_opt(|syn_file| {
            syn_file.items.into_iter()
                .map(|p| {
                    error!("parsing {:?}", &SynHelper::get_str(&p));
                    p
                })
                .filter(|i| matches!(i, Item::Mod(_))).next()
        })
        .as_mut()
        .flat_map_opt(|item_mod| {
            info!("Parsing {:?}", &SynHelper::get_str(&item_mod));
            let program_src = user_program_src!();
            parse_module_into_container(&program_src, item_mod, &mut module_parser)
                .or_else(|| {
                    error!("Could not parse {:?}", &SynHelper::get_str(&item_mod));
                    None
                })
                .as_mut()
                .map(|parse_container| {
                    if let Item::Mod(item_mod) = item_mod {
                        let item_mod_key = ParseContainer::get_bean_definition_key_item_mod(&item_mod).unwrap();
                        info!("{:?} are keys.", &parse_container.modules.keys());
                        parse_container.modules.remove(&item_mod_key)
                            .as_mut()
                            .map(|item_mod| {
                                info!("Parsing item mode in dfactory");
                                ItemModParser::parse_item(&program_src, parse_container, item_mod, vec![item_mod.ident.clone().to_string()], &mut module_parser);
                                parse_container.modules.insert(item_mod_key, item_mod.clone());
                            });
                    }

                    let p = Box::new(ProfileProfileTreeModifier::new(&parse_container.injectable_types_builder));
                    let mut profile_tree = ProfileTreeBuilder::build_profile_tree(&mut parse_container.injectable_types_builder,
                                                                                  vec![p], &mut parse_container.provided_items);

                    info!("Build profile tree: {:?}.", &profile_tree);
                    let d = DelegatingTokenProvider::new(&mut profile_tree);
                    let mut generated: TokenStream = d.generate_token_stream();

                    let mut ts: TokenStream = quote! {
                        use proc_macro2::TokenStream;
                        use syn::Item;
                        use quote::quote;
                        use dfactory_dcodegen_shared::MutableModuleModifier;
                        use syn::ImplItem;
                    };

                    ts.extend(generated);
                    info!("generated: {:?}.", SynHelper::get_str(&ts));
                    ts
                })
        })
        .map(|ts| ts.to_string())
        .flat_map_opt(|tokens| {
            CrateWriter::write_lib_rs_crate(
                "knockoff_delegator_factories",
                "0.1.5",
                &project_directory_path!().join("target"),
                FactoriesParser::default_phase_deps(&Phase::DFactory)
                    .as_table().or(Some(&Table::new())).unwrap(),
                &tokens
            );
            Some(tokens)
        })
        .or_else(|| {
            None::<String>
        })
}

#[test]
pub fn test_load() {
    info!("{:?}", write_d_factory_crate());
}

