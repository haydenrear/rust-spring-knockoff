use std::{env, fs};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::fmt::Error;
use std::path::Path;
use quote::{quote, ToTokens};
use syn::{Attribute, Item, ItemFn};
use knockoff_logging::{initialize_log, initialize_logger, use_logging, create_logger_expr};
use crate::authentication_type::AuthenticationType;
use crate::initializer::Initializer;
use crate::field_aug::FieldAug;

use_logging!();
initialize_log!();

use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

pub struct LibParser;

impl LibParser {

    pub fn do_codegen(in_dir_file: &str, initializer: bool, out_file: &str) {

        let mut codegen_items = Self::gen_codegen_items(initializer).codegen;

        log_message!("Found {} codegen items.", codegen_items.len());

        let mut type_id_map: HashMap<String, Box<dyn CodegenItem>> = codegen_items.iter()
            .map(|c| (c.get_unique_id(), c.clone_dyn_codegen()))
            .collect();

        let mut to_write_codegen = Self::parse_codegen(in_dir_file, codegen_items);

        for types in type_id_map.iter() {
            if !to_write_codegen.contains_key(types.0)  {
                let string = types.1.default_codegen();
                to_write_codegen.insert(types.0.clone(), string);
            }
        }

        let codegen = to_write_codegen.values().into_iter()
            .map(|s| s.clone())
            .collect::<Vec<String>>().join("");

        Self::write_codegen(&codegen, out_file);

    }

    fn parse_codegen(in_dir_file: &str, mut codegen_items: Vec<Box<dyn CodegenItem>>) -> HashMap<String, String> {
        let mut to_write_codegen: HashMap<String, String> = Self::parse_syn(in_dir_file)
            .iter()
            .flat_map(|syn_file| {
                syn_file.items.iter()
            })
            .flat_map(|item| {
                codegen_items.iter()
                    .filter(move |c| c.supports(&item))
                    .map(|codegen_item| {
                        (codegen_item.get_unique_id(), codegen_item)
                    })
                    .map(move |c_item| (c_item.0, c_item.1.get_codegen(&item)))
                    .flat_map(|codegen| {
                        if codegen.1.is_none() {
                            return vec![]
                        } else {
                            return vec![(codegen.0, codegen.1.unwrap())]
                        }
                    })
            })
            .collect();
        to_write_codegen
    }


    pub fn parse_syn(in_dir_file: &str) -> Option<syn::File> {
        let in_path = Path::new(in_dir_file);
        if in_path.exists() {
            return fs::read_to_string(in_path).ok().and_then(|mut in_file_result| {
                syn::parse_file(in_file_result.as_str()).ok().or(None)
            })
        }
        None
    }

    pub fn gen_codegen_items(initializer: bool) -> Codegen {
        if initializer {
            return Codegen {
                codegen: vec![Box::new(Initializer::new()), Box::new(FieldAug::new()), Box::new(AuthenticationType::new())]
            };
        } else {
            return Codegen {
                codegen: vec![Box::new(FieldAug::new()), Box::new(AuthenticationType::new())]
            };
        }
    }

    pub fn write_codegen(codegen_out: &str, codegen: &str) {
        let out_path = Path::new(&env::var("OUT_DIR").unwrap())
            .join(codegen);
        File::create(out_path).map(|mut out_file_create| {
            out_file_create.write(Self::get_imports().as_bytes())
                .unwrap();
            out_file_create.write(codegen_out.as_bytes())
                .unwrap();
        }).or_else(|err| {
            Ok::<(), Error>(())
        }).unwrap();
    }

    pub fn get_imports() -> String {
        let quoted = quote! {
            use derive_syn_parse::Parse;
            use module_macro_shared::module_macro_shared_codegen::{ContextInitializer, FieldAugmenter};
            use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf, TypeArray, ItemEnum};
            use syn::parse::Parser;
            use quote::{quote, ToTokens};
        };
        quoted.to_string()
    }
}

pub trait CodegenItem {
    fn supports(&self, item: &Item) -> bool;
    fn get_codegen(&self, tokens: &Item) -> Option<String>;
    fn default_codegen(&self) -> String;
    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem>;
    fn get_unique_id(&self) -> String;
}

pub struct Codegen {
    pub codegen: Vec<Box<dyn CodegenItem>>
}