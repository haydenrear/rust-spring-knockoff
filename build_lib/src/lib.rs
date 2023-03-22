#![feature(io_error_more)]

use std::{env, fs};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::convert::identity;
use std::ffi::{c_long, OsStr, OsString};
use std::fmt::{Debug, Error, Formatter};
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
use codegen_utils::env::{get_project_base_dir, get_project_dir};
use codegen_utils::{parse, project_directory, syn_helper};
use codegen_utils::syn_helper::SynHelper;
use codegen_utils::walk::DirectoryWalker;
use knockoff_logging::{create_logger_expr, initialize_log, initialize_logger, use_logging};

use_logging!();
initialize_logger!(TextFileLoggerImpl, StandardLogData, concat!(project_directory!(), "log_out/build_lib.log"));
initialize_log!();

#[test]
fn do_test() {
    replace_modules(
        Some(get_project_dir("delegator_test/src").as_str()),
        vec![get_project_base_dir().as_str()]
    );
}

pub struct ModuleReplacer {
    modules: Vec<Module>,
}

#[derive(Default, Clone)]
pub struct Module {
    pub modules: Vec<Module>,
    pub mod_items: Vec<Item>,
    pub is_head: bool,
    pub identifier: Option<Ident>,
    pub mod_item: Option<ItemMod>
}

impl Module {
    fn debug_module(&self) {
        self.identifier.clone().map(|id| {
            log_message!("Debugging module with name {} and {} modules and {} mod items with head {}.", id, self.modules.len(), self.mod_items.len(), self.is_head);
        }).or_else(|| {
            log_message!("Debugging module with no name and {} modules and {} mod items with head {}.", self.modules.len(), self.mod_items.len(), self.is_head);
            None
        });
        self.modules.iter().for_each(|module| {
            log_message!("Here is other module: ");
            module.debug_module();
        })
    }
}

/// This can probably be replaced with just parsing the modules and generating the
/// code based on it, instead of having to relocate the modules. Then, code complete will
/// be provided without any further.
pub fn replace_modules(base_env: Option<&str>, rerun_files: Vec<&str>) {

    log_message!("Starting to replace modules.");

    if base_env.is_none() {
        return;
    }

    log_message!("Continuing to replace modules.");

    Module::parse_syn(base_env)
        .map(|lib_file|
            Module::do_parse(
                Module::parse_modules(base_env.unwrap())
            )
        );

    rerun_files.iter().for_each(|rerun_file| {
        println!("cargo:rerun-if-changed={}", rerun_file.to_string().as_str());
    });

}

impl Module {

    fn parse_modules(base_env: &str) -> Module {
        let buf = Self::get_lib_file_path(base_env);
        let string = OsString::from("main.rs");
        let path = buf.file_name()
            .or(Some(&string))
            .unwrap()
            .to_str().or(Some("main.rs"))
            .unwrap();

        Self::parse_syn(Some(base_env))
            .map(|syn_file| Self::parse_module_from_syn_file(&syn_file, base_env, path, None, &buf))
            .or(None)
            .unwrap()

    }

    fn parse_module_from_file(file_to_parse: &mut File, base_dir: &str, module_file: &str, id: Option<Ident>, syn_file_buf: &PathBuf) -> Module {
        SynHelper::parse_syn_file(file_to_parse)
            .map(|syn_file| {
                log_message!("Successfully parsed module syn file.");
                Self::parse_module_from_syn_file(&syn_file, base_dir, module_file, id, syn_file_buf)
            })
            .or(None)
            .unwrap()
    }

    fn is_main_or_lib(module_file_name: &str) -> bool {
        module_file_name == "main.rs" || module_file_name == "main.rs"
    }

    fn parse_module_from_syn_file(syn_file: &syn::File, base_dir: &str, module_file_name: &str, id: Option<Ident>, buf: &PathBuf) -> Module {

        let mod_items = Self::parse_items(&syn_file);
        let mut modules = Self::parse_submodules(&syn_file, base_dir, module_file_name, buf);

        let mut module = Module {
            modules: vec![],
            mod_items,
            identifier: id,
            is_head: if Self::is_main_or_lib(module_file_name) { true } else { false },
            mod_item: None,
        };

        if Self::is_main_or_lib(module_file_name) && module.modules.len() > 1 {
            panic!("Only one main module allowed with module macro");
        } else if Self::is_main_or_lib(module_file_name) {
            let first_mod = modules.remove(0);
            module.identifier = first_mod.identifier;
            module.mod_items = first_mod.mod_items;
            module.modules = first_mod.modules;
            module.mod_item = first_mod.mod_item;
        } else {
            module.modules = modules;
        }

        module.debug_module();

        module

    }

    fn get_lib_file_path(base_env: &str) -> PathBuf {
        let mut path = Path::new(base_env).join("main.rs");
        if !path.exists() {
            path = Path::new(base_env).join("main.rs");
        }
        path
    }

    fn parse_items(syn_file: &syn::File) -> Vec<Item> {
        syn_file.items.iter().flat_map(|item| {
            log_message!("Parsed in syn file {}.", item.to_token_stream().to_string().as_str());
            match item {
                Item::Mod(item_mod) => {
                    vec![]
                }
                other => {
                    vec![item.to_owned()]
                }
            }
        }).collect::<Vec<Item>>()
    }

    fn parse_submodules(syn_file: &syn::File, base_dir: &str, module_or_file_name: &str, syn_file_buf: &PathBuf) -> Vec<Module> {
        syn_file.items.iter().flat_map(|item| {
            log_message!(
                "Parsed in syn file {}.",
                item.to_token_stream().to_string().as_str()
            );
            match item {
                Item::Mod(item_mod) => {
                    if item_mod.ident.to_string().as_str() != module_or_file_name {
                        return vec![Self::parse_module(base_dir, item_mod, item_mod.ident.to_string().as_str(), syn_file_buf)];
                    }
                    vec![]
                }
                _ => {
                    vec![]
                }
            }
        }).collect::<Vec<Module>>()
    }

    fn parse_module(base_dir: &str, item_mod: &ItemMod, module_name_or_file: &str, syn_file_buf: &PathBuf) -> Module {
        log_message!("successfully parsed mod file.");
        let ident = item_mod.ident.clone();
        let found = Self::walk_find_mod_file(base_dir, ident.to_string().as_str(), syn_file_buf)
            .map(|mut item| {
                log_message!("parsed mod file for {}.", item_mod.to_token_stream().to_string().as_str());
                item.1.ok()
                    .map(|mut item_file| Self::parse_module_from_file(
                        &mut item_file,
                        base_dir,
                        module_name_or_file,
                        Some(ident.clone()),
                        syn_file_buf
                    ))
            })
            .flatten()
            .or_else(|| {

                let (module_items, containing_modules)
                    = Self::create_get_module_items(
                        base_dir, item_mod, module_name_or_file, syn_file_buf
                    );

                log_message!(
                    "Creating module for {} with {} module items and {} modules.",
                    item_mod.ident.to_string().as_str(), module_items.len(), containing_modules.len()
                );

                Some(Module {
                    identifier: Some(ident),
                    modules: containing_modules,
                    mod_items: module_items,
                    is_head: false,
                    mod_item: Some(item_mod.clone()),
                })
            });

        found.unwrap()
    }

    fn create_get_module_items(base_dir: &str, item_mod: &ItemMod, module_name_or_file: &str, syn_file_buf: &PathBuf) -> (Vec<Item>, Vec<Module>) {
        let mut module_items = vec![];
        let mut containing_modules = vec![];

        item_mod.content.clone().map(|item_content| {
            item_content.1.iter().for_each(|items_found| {
                match items_found {
                    Item::Mod(item_mod_again) => {
                        log_message!(
                            "Found {} item mod in create get module items and {} is module name or file.",
                            item_mod_again.ident.clone().to_string(), module_name_or_file.clone().to_string()
                        );
                        if item_mod_again.ident.to_string().as_str() != module_name_or_file {
                            let next_mod = Self::parse_module(
                                base_dir, item_mod_again,
                                item_mod_again.ident.to_string().as_str(),
                                syn_file_buf
                            );
                            containing_modules.push(next_mod);
                        }
                    }
                    other => {
                        module_items.push(other.to_owned());
                    }
                }
            });
        });

        (module_items, containing_modules)
    }

    fn parse_syn(base_env: Option<&str>) -> Option<syn::File> {
        parse::open_file(base_env.unwrap(), "lib.rs")
            .or_else(|_| parse::open_file(base_env.unwrap(), "main.rs"))
            .map(|mut file| SynHelper::parse_syn_file(&mut file))
            .map_err(|err| {
                log_message!("Error opening main.rs or lib.rs file: {}.", err.to_string());
                err
            })
            .ok()
            .flatten()
    }

    fn log_module(&self) {
        self.modules.iter().for_each(|m| {
            m.log_module();
        });
        self.mod_items.iter().for_each(|m| {
            log_message!("Found {} as mod item in module.", m.to_token_stream().to_string().as_str());
        });
    }

    fn do_parse(mut modules: Module) {
        let out_dir = env::var_os("OUT_DIR")
            .or(Some(OsString::from(concat!(project_directory!(), "test_out")))).unwrap();
        let dest_path = Path::new(&out_dir).join("spring-knockoff.rs");
        let out_path = dest_path.to_str().unwrap();
        log_message!("Writing output to {}.", out_path);
        if File::open(dest_path.clone()).is_ok() {
            fs::remove_file(&dest_path.clone())
                .unwrap();
        }

        log_message!("Doing final parse.");

        modules.debug_module();

        File::create(&dest_path)
            .unwrap();

        for mut next_mod in modules.modules.clone().iter() {
            let mut next_mod_owned = next_mod.to_owned();
            next_mod_owned = Self::do_parse_recursive(next_mod_owned);
            modules.add_mod(next_mod_owned.to_owned());
        }

        modules.process_lib_main_mod(dest_path);

    }


    fn do_parse_recursive(mut modules: Module) -> Module {
        for mut next_mod in modules.modules.clone().iter() {
            let mut next_mod = next_mod.to_owned();
            next_mod = Self::do_parse_recursive(next_mod);
            modules.add_mod(next_mod.to_owned());
        }
        modules.to_owned()
    }

    /// At this point, the prev_mod will have already put all of it's modules into the mod_items.
    fn add_mod(&mut self, mut prev_mod: Module) {
        log_message!("Adding {} mod items.", prev_mod.mod_items.len());
        log_message!("Module now has {} items.", self.mod_items.len());

        for mut next_mod in prev_mod.mod_items.clone().iter() {
            if prev_mod.identifier.is_none() && self.identifier.is_none() {
                log_message!("was none when parsing {}", self.identifier.clone().unwrap());
                continue;
            } else if (prev_mod.identifier.is_none() && self.identifier.is_none()) {
                log_message!("Warning: both identifiers are none!");
                continue;
            }
            self.add_item_mod(next_mod.clone(), prev_mod.identifier.clone().unwrap())
        }

        log_message!("module now has {} items.", self.mod_items.len());
    }

    fn add_item_mod(&mut self, item: Item, ident: Ident) {
        let index = self.get_item_mod_index(item.clone(), ident.clone());
        if index != -1 {
            let mut mod_item_to_update = self.mod_items.remove(index as usize);
            match mod_item_to_update {
                Item::Mod(ref mut item_mod) => {
                    item_mod.content.as_mut().map(|mut i| {
                        log_message!("Adding to item mod!");
                        i.1.push(item.clone());
                    });
                    log_message!("Adding mod.");
                    self.mod_items.push(Item::Mod(item_mod.clone()));
                }
                _ => {
                    self.mod_items.push(mod_item_to_update);
                }
            }
        } else {
            let mut new_mod = ItemMod {
                attrs: vec![],
                vis: Visibility::Public(VisPublic{ pub_token: Default::default() }),
                mod_token: Default::default(),
                ident,
                content: Some((Brace::default(), vec![item])),
                semi: None,
            };
            self.mod_items.push(Item::Mod(new_mod));
        }
    }

    fn get_item_mod_index(&mut self, item: Item, ident: Ident) -> i32 {
        log_message!("Adding {} to module {}.", item.clone().to_token_stream().to_string().as_str(), self.identifier.clone().unwrap().to_string().as_str());
        let mut count = 0;
        for mut prospective_mod_item in &self.mod_items {
            match prospective_mod_item {
                Item::Mod(item_mod) => {
                    if item_mod.ident == ident {
                        return count;
                    }
                }
                _ => {
                }
            };
            count += 1;
        }
        log_message!("Index did not exist in item for {}!", ident.to_string().as_str());
        -1
    }

    fn process_lib_main_mod(&mut self, dest_path: PathBuf) {
        let mut existing = fs::read_to_string(dest_path.clone())
            .unwrap();

        log_message!("{} is starting existing", existing.as_str());
        log_message!("There are {} other items.", self.mod_items.len());
        log_message!("Here is the lib.");
        if self.mod_item.clone().is_none() {
            log_message!("Could not find mod item.");
        } else {
            let mut final_mod_item = self.mod_item.clone().unwrap();

            final_mod_item.content = Some((Brace::default(), self.mod_items.clone()));

            final_mod_item = Self::remove_cfg_for_codegen(&mut final_mod_item);
            Self::write_to_module_file(&mut existing, &final_mod_item);

            fs::write(dest_path.clone(), existing)
                .unwrap();
        }


    }

    fn write_to_module_file<T: ToTokens>(mut existing: &mut String, mod_created: &T) {
        log_message!("{} is the finished module", mod_created.to_token_stream().to_string().as_str());
        existing.push_str(mod_created.to_token_stream().to_string().as_str());
    }

    fn remove_cfg_for_codegen(x: &mut ItemMod) -> ItemMod {
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
            x.attrs.remove(cfg_attr);
        }

        x.to_owned()

    }

    fn create_module(
        mut item_mod: ItemMod, items: Vec<Item>
    ) -> ItemMod {
        log_message!("Using {}.", item_mod.ident.to_token_stream().to_string().as_str());
        item_mod.content = Some((Brace::default(), items));
        item_mod
    }

    fn get_default() -> TokenStream {
        let ts = quote! {
        struct DefaultVal;
    };
        ts
    }

    fn walk_find_mod_file(base_dir: &str, module_name: &str, parent_buf: &PathBuf) -> Option<(PathBuf, Result<File, std::io::Error>)> {
        let dirs_with_name = DirectoryWalker::walk_directory(module_name, base_dir);

        if dirs_with_name.len() == 1 {
            let correct_buf = dirs_with_name[0].to_owned();
            return Some((correct_buf.clone(), File::open(correct_buf)));
        } else if dirs_with_name.len() == 0 {
            return None
        }

        Self::find_correct_buf(dirs_with_name, parent_buf)
            .map(|correct_buf| Some((correct_buf.clone(), File::open(correct_buf))))
            .flatten()
            .or(None)

    }

    fn find_correct_buf(bufs: Vec<PathBuf>, parent_buf: &PathBuf) -> Option<PathBuf> {
        bufs.iter().filter(|b| {
            parent_buf.to_str().map(|parent_buf| {
                b.to_str().map(|child_buf| {
                    parent_buf.contains(child_buf)
                })
            }).or_else(|| {
                panic!("Parent path could not be unwrapped!");
                None
            }).is_some()
        })
            .max_by(|first, second| {
                (first.to_str().unwrap().len() as i32).cmp(&(second.to_str().unwrap().len() as i32))
            })
            .map(|c| c.to_owned())
    }

}

