#![feature(io_error_more)]

use std::{env, fs};
use std::borrow::Borrow;
use std::ffi::{OsStr, OsString};
use std::fmt::Error;
use std::fs::{DirEntry, File, ReadDir};
use std::io::{ErrorKind, Read, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::ptr::write;
use quote::__private::TokenStream;
use quote::quote;
use syn::__private::{Span, ToTokens};
use syn::{braced, Fields, Ident, Item, ItemImpl, ItemMod, ItemStruct, parse_macro_input, Token, token, Visibility, VisPublic};
use syn::__private::quote::__private::push_div_eq_spanned;
use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use codegen_utils::parse;
use codegen_utils::walk::DirectoryWalker;
use knockoff_logging::{create_logger_expr, initialize_log, initialize_logger, use_logging};

use_logging!();
initialize_logger!(TextFileLoggerImpl, StandardLogData, "/Users/hayde/IdeaProjects/rust-spring-knockoff/log_out/build_lib.log");
initialize_log!();

pub struct ModuleReplacer {
    modules: Vec<Module>,
}

#[derive(Default, Clone)]
pub struct Module {
    pub modules: Vec<Module>,
    pub mod_item: Vec<ItemMod>,
    pub other_items: Vec<Item>,
    pub path: PathBuf,
    pub is_head: bool
}

pub fn replace_modules(base_env: Option<&str>, rerun_files: Vec<&str>) {
    if base_env.is_none() {
        return;
    }
    Module::parse_syn(base_env)
        .map(|lib_file| Module::do_parse(rerun_files, Module::parse_modules(base_env.unwrap())));
}

impl Module {

    fn new(item_mod: ItemMod, base_project: &str) -> Self {
        Self {
            modules: vec![],
            mod_item: vec![item_mod],
            other_items: vec![],
            path: DirectoryWalker::walk_directory(item_mod.ident.to_string().as_str(), base_project).unwrap_or_else(|| {
                log_message!("Failed to open directory for module {}.", item_mod.ident.to_string().as_str());
                PathBuf::default()
            }),
            is_head: false,
        }
    }

    fn parse_modules(base_env: &str) -> Module {

        let modules = Self::parse_syn(Some(base_env))
            .map(|syn_file| Self::parse_module_from_syn_file(syn_file, base_env))
            .or(Some(vec![]))
            .unwrap();

        let mut path = Path::new(base_env).join("lib.rs");
        if !path.exists() {
            path =  Path::new(base_env).join("main.rs");
        }

        Self::parse_syn(Some(base_env))
            .map(|syn_file| Self::parse_single_layer_module(syn_file, base_env))
            .map(|head_items| {
                Module {
                    modules,
                    mod_item: head_items.1,
                    other_items: head_items.0,
                    path,
                    is_head: true
                }
            })
            .unwrap()
    }

    fn parse_module_from_file(file_to_parse: &mut File, base_dir: &str) -> Vec<Module> {
        parse::parse_syn_file(file_to_parse)
            .map(|syn_file| {
                Self::parse_module_from_syn_file(syn, base_dir)
            })
            .or(Some(vec![]))
            .unwrap()
    }

    fn parse_single_layer_module(syn_file: syn::File, base_dir: &str) -> (Vec<Item>, Vec<ItemMod>) {
        let item_mods = Self::parse_single_layer_item_mod(syn_file, base_dir);

        let item_mod = item_mods.iter().flat_map(|item| {
            match item{
                Item::Mod(item_mod) => {
                    vec![item_mod.clone()]
                }
                _ => {
                    vec![]
                }
            }
        }).collect::<Vec<ItemMod>>();

        let other_items = item_mods.iter().flat_map(|item| {
            match item{
                Item::Mod(_) => {
                    vec![]
                }
                other => {
                    vec![other]
                }
            }
        }).collect::<Vec<Item>>();

        (other_items, item_mod)

    }

    fn parse_single_layer_item_mod(syn_file: syn::File, base_dir: &str) -> Vec<Item> {
        syn_file.items.iter().flat_map(|item| {
            match item {
                Item::Mod(item_mod)  => {
                    if Self::walk_find_mod_file(
                        base_dir,
                        item_mod.ident.to_string().as_str()
                    ).is_none() {
                        return Self::parse_single_layer_item_mod(syn_file, base_dir).iter();
                    }
                    vec![].iter()
                }
                other => {
                    vec![other]
                }
            }
        }).collect::<Vec<Item>>()
    }

    fn parse_module_from_syn_file(syn_file: syn::File, base_dir: &str) -> Vec<Module> {
        syn_file.items.iter().flat_map(|item| {
            match item {
                Item::Mod(item_mod) => {
                    Self::parse_module_from_separate_file(base_dir, item_mod)
                }
                _ => {
                    vec![]
                }
            }
        }).collect::<Vec<Module>>()
    }

    fn parse_module_from_separate_file(base_dir: &str, item_mod: &ItemMod) -> Vec<Module> {
        let found = Self::walk_find_mod_file(base_dir, item_mod.ident.to_string().as_str())
            .map(|item| item.1.ok()
                .map(|&mut item_file| Self::parse_module_from_file(item_file, base_dir))
            )
            .flatten()
            .or(Some(vec![]));

        found.unwrap()
    }

    fn parse_syn(base_env: Option<&str>) -> Option<syn::File> {
        parse::open_file(base_env.unwrap(), "lib.rs")
            .or_else(|| parse::open_file(base_env.unwrap(), "main.rs"))
            .map(|mut file| parse::parse_syn_file(&mut file))
            .map_err(|err| {
                log_message!("Error opening lib.rs file: {}.", err.to_string());
                err
            })
            .ok()
            .flatten()
    }

    fn create_order(modules: Module) -> Vec<Module> {
        /// Make sure that the last module is first so that they can be inserted into the containing module
        if modules.modules.len() != 0 {
            let mut this_module = modules.modules.iter()
                .flat_map(|m| Self::create_order(m.to_owned()).iter())
                .collect::<Vec<Module>>();
            modules.mod_item.map(|m| this_module.push(m));
            return this_module;
        }

        if !modules.is_head {
            log!(LogLevel::Error, "First module was not head! Error parsing module tree.".to_string(), "6".to_string());
        }

        vec![modules]
    }

    fn find_mod_content_index(item_mod: &ItemMod, name: &str) -> usize {
        let mut counter = 0;
        item_mod.content.map(|c| {
            let items = c.1;
            if items.len() == 0 {
                counter = -1;
            } else {
                for i in items.iter() {
                    match i {
                        Item::Mod(item_mod) => {
                            if item_mod.ident.to_string().as_str() == name {
                                break;
                            }
                        }
                        _ => {}
                    }
                    counter += 1;
                }
            }
        }).or_else(|| {
            counter = -1;
            None
        });
        counter
    }

    fn do_parse(rerun_files: Vec<&str>, modules: Module) {
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join("spring-knockoff.rs");
        if File::open(dest_path.clone()).is_ok() {
            fs::remove_file(&dest_path.clone())
                .unwrap();
        }

        File::create(&dest_path)
            .unwrap();

        let mut prev = None;

        for mut x in Self::create_order(modules) {

            if prev.is_none() {
                prev = Some(x);
                continue;
            }

            let prev_mod_module = prev.unwrap();

            for prev_mod in prev_mod_module.modules.iter() {

                prev_mod.mod_item.iter()
                    .flat_map(|c| c.content
                        .map(|content| content.1.iter()
                            .map(|content| (
                                content,
                                Self::find_mod_content_index(m, c.ident.to_string().as_str()),
                                c.ident.to_string().clone()
                            ))
                            .collect()
                        )
                        .or(Some(vec![])).unwrap()
                    )
                    .for_each(|m| {
                        let mod_index = m.1;

                        if mod_index != -1 {
                            x.add_item_to_module(m.2.as_str(), mod_index, m.0.clone())
                        }

                });

            }
        }


        Self::process_lib_main_mod(dest_path, &mut prev);

        rerun_files.iter().for_each(|rerun_file| {
            print!("cargo:rerun-if-changed={}", rerun_file);
        })
    }

    fn add_item_to_module(&mut self, item_mod_name: &str, mod_index: usize, item: Item) {
        let mut index = 0;
        let mut contains = false;
        for mod_item in self.mod_item.iter() {
            if mod_item.ident.to_string().as_str() == item_mod_name {
                contains = true;
                break;
            }
            index += 1;
        }
        if contains {
            self.mod_item[index].content[mod_index] = item;
        }
    }

    fn process_lib_main_mod(dest_path: PathBuf, prev: &mut Option<Module>) {
        if prev.is_some() {
            let mut existing = fs::read_to_string(dest_path.clone())
                .unwrap();
            prev.unwrap().mod_item.iter().for_each(|mut mod_created| {
                Self::remove_cfg_for_codegen(&mut mod_created);
                Self::write_to_module_file(&mut existing, mod_created);
            });
            prev.unwrap().other_items.iter().for_each(|mut mod_created| {
                log_message!("{} is the finished module", mod_created.to_token_stream().to_string().as_str());
                existing.push_str(mod_created.to_token_stream().to_string().as_str());
            });
            fs::write(dest_path.clone(), existing)
                .unwrap();
        }
    }

    fn write_to_module_file<T: ToTokens>(mut existing: &mut String, mut mod_created: &T) {
        log_message!("{} is the finished module", mod_created.to_token_stream().to_string().as_str());
        existing.push_str(mod_created.to_token_stream().to_string().as_str());
    }

    fn remove_cfg_for_codegen(x: &mut ItemMod) {
        let mut cfg_attr = 0;
        let mut counter = 0;

        for attr in x.attrs.clone().iter() {
            if attr.to_token_stream().to_string().as_str().contains("module_attr") {
                log_message!("found attr on main module: {}.", attr.tokens.to_string().as_str());
            } else if attr.to_token_stream().to_string().as_str().contains("cfg")
                && attr.to_token_stream().to_string().as_str().contains("springknockoff") {
                cfg_attr = counter;
            }
            counter += 1;
        }

        if cfg_attr != 0 {
            module.attrs.remove(cfg_attr);
        }

    }

    fn create_module(
        mut item_mod: ItemMod, items: Vec<Item>
    ) -> ItemMod {
        log_message!("Using {}.", module.ident.to_token_stream().to_string().as_str());
        item_mod.content = Some((Brace::default(), items));
        item_mod
    }

    fn get_default() -> TokenStream {
        let ts = quote! {
        struct DefaultVal;
    };
        ts
    }

    fn walk_find_mod_file(base_dir: &str, module_name: &str) -> Option<(PathBuf, Result<File, std::io::Error>)> {
        DirectoryWalker::walk_directory(module_name, base_dir)
            .filter(|dir| dir.exists())
            .map(|dir| {
                (dir.clone(), File::open(dir))
            })
    }

}

