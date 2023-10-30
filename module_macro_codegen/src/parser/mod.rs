use std::{env, fs};
use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::fmt::Error;
use std::path::Path;
use std::sync::Arc;
use quote::{quote, ToTokens};
use syn::{Attribute, Item, ItemFn};
use crate::authentication_type::AuthenticationTypeCodegen;
use crate::codegen_items;
use crate::initializer::Initializer;
use crate::field_aug::FieldAug;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("parser.rs");
use crate::module_extractor::ModuleParser;

codegen_items!(
    (AuthenticationTypeCodegen, AuthenticationType),
    (FieldAug, FieldAug),
    (ModuleParser, Module),
    (Initializer, ContextInitializer)
);

#[test]
fn test() {

}

pub struct LibParser;

impl LibParser {

    pub fn do_codegen(in_dir_file: &str, out_file: &str) {

        let mut codegen_items = Self::gen_codegen_items().codegen;

        log_message!("Found {} codegen items.", codegen_items.len());

        let mut type_id_map: HashMap<String, CodegenItemType> = codegen_items.iter()
            .map(|c| {
                log_message!("Found codegen item with ID: {}", c.get_unique_id().as_str());
                (c.get_unique_id(), c.clone())
            })
            .collect();

        let mut to_write_codegen = Self::parse_codegen(in_dir_file);

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

    fn parse_codegen(in_dir_file: &str) -> HashMap<String, String> {
        let flatten = Self::parse_codegen_items(in_dir_file);
        let mut to_write_codegen: HashMap<String, String> = flatten
            .iter()
            .flat_map(|item|
                item.get_codegen()
                    .map(|codegen| (item.get_unique_id(), codegen))
            )
            .collect();

        to_write_codegen
    }

    pub fn parse_codegen_items(in_dir_file: &str) -> Vec<CodegenItemType> {
        let flatten = Self::parse_syn(in_dir_file)
            .iter()
            .flat_map(|syn_file| get_codegen_item(&syn_file.items.clone()))
            .collect::<Vec<CodegenItemType>>();
        flatten
    }

    pub fn parse_syn(in_dir_file: &str) -> Option<syn::File> {
        let in_path = Path::new(in_dir_file);
        if in_path.exists() {
            return fs::read_to_string(in_path).ok()
                .and_then(|mut in_file_result|
                    syn::parse_file(in_file_result.as_str()).ok().or(None)
                )
        }
        None
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

    fn supports_item(item: &Vec<Item>) -> bool where Self: Sized;

    fn supports(&self, item: &Vec<Item>) -> bool;

    fn get_codegen(&self) -> Option<String>;

    fn default_codegen(&self) -> String;

    fn get_unique_id(&self) -> String;

    fn get_unique_ids(&self) -> Vec<String> {
        vec![self.get_unique_id()]
    }

}

#[derive(Default, Clone)]
pub struct CodegenItems {
    pub codegen: Vec<CodegenItemType>
}
