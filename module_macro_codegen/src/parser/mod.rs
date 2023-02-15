use std::{env, fs};
use std::fs::File;
use std::io::Write;
use std::fmt::Error;
use std::path::Path;
use quote::{quote, ToTokens};
use syn::{Attribute, Item, ItemFn};
use crate::initializer::Initializer;
use crate::field_aug::FieldAug;

pub struct LibParser;

impl LibParser {

    pub fn do_codegen(in_dir_file: &str, log_file: &mut File, initializer: bool, out_file: &str) {

        let mut codegen_items = Self::gen_codegen_items(initializer).codegen;
        let to_write_codegen = Self::parse_syn(in_dir_file, log_file)
            .iter()
            .flat_map(|syn_file| {
                syn_file.items.iter()
            })
            .flat_map(|item| {
                codegen_items.iter().filter(move |c| c.supports(&item))
                    .map(move |c_item| c_item.get_codegen(&item))
            })
            .flatten()
            .collect::<String>();


        Self::write_codegen(to_write_codegen.as_str(), out_file);

    }


    pub fn parse_syn(in_dir_file: &str, log_file: &mut File) -> Option<syn::File> {
        let in_path = Path::new(in_dir_file);
        if in_path.exists() {
            return fs::read_to_string(in_path).ok().and_then(|mut in_file_result| {
                log_file.write("in file exists".as_bytes()).unwrap();
                syn::parse_file(in_file_result.as_str()).ok().or(None)
            })
        }
        None
    }

    pub fn gen_codegen_items(initializer: bool) -> Codegen {
        if initializer {
            return Codegen {
                codegen: vec![Box::new(Initializer {}), Box::new(FieldAug {})]
            };
        } else {
            return Codegen {
                codegen: vec![Box::new(FieldAug {})]
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
}

pub struct Codegen {
    pub codegen: Vec<Box<dyn CodegenItem>>
}