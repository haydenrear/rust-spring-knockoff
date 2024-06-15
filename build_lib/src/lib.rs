#![feature(io_error_more)]

use std::{env, fs};
use std::borrow::Borrow;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use quote::__private::TokenStream;
use quote::quote;
use syn::__private::ToTokens;
use syn::{Attribute, Ident, Item, ItemMod, Visibility, VisPublic};
use syn::token::Brace;
use codegen_utils::{parse, project_directory};
use codegen_utils::syn_helper::SynHelper;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::{get_build_project_dir, get_project_base_build_dir};

import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/build_lib.log"));

#[test]
fn do_test() {
    replace_modules(
        Some(get_build_project_dir("delegator_test/src").as_str()),
        vec![get_project_base_build_dir().as_str()]
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
    pub mod_item: Option<ItemMod>,
    pub attrs: Vec<Attribute>
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

pub fn replace_modules(base_env: Option<&str>, rerun_files: Vec<&str>) {

    log_message!("Starting to replace modules.");

    if base_env.is_none() {
        return;
    }

    log_message!("Continuing to replace modules.");

    Module::parse_syn(base_env)
        .map(|lib_file| {
            Module::do_parse(
                Module::parse_modules(base_env.unwrap())
            )
        });

    rerun_files.iter().for_each(|rerun_file| {
        println!("cargo:rerun-if-changed={}", rerun_file.to_string().as_str());
    });

}

impl Module {

    fn parse_modules(base_env: &str) -> Module {
        info!("Parsing modules!");
        let buf = Self::get_lib_file_path(base_env);
        let string = OsString::from("main.rs");
        let path = buf.file_name()
            .or(Some(&string))
            .unwrap()
            .to_str()
            .or(Some("main.rs"))
            .unwrap();

        info!("Found next {} path.", path);

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
        module_file_name == "main.rs" || module_file_name == "lib.rs"
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
            attrs: vec![],
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
                log_message!("Walked to find mod file for {}.", item_mod.to_token_stream().to_string().as_str());
                info!("{:?} is item path and {:?} is result from walking to find it.", &item.0.to_str(), &item.1);
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
                    attrs: item_mod.attrs.clone(),
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
                            "Found {} item mod in create get module items and {} is module name or file where mod was declared.",
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
            info!("Parsing next modue: {:?}", next_mod.identifier.as_ref().unwrap().to_string().as_str());
            let mut next_mod = next_mod.to_owned();
            next_mod = Self::do_parse_recursive(next_mod);
            modules.add_mod(next_mod.to_owned());
        }
        modules.to_owned()
    }

    /// At this point, the prev_mod will have already put all of it's modules into the mod_items.
    fn add_mod(&mut self, mut prev_mod: Module) {
        info!("Adding module: {:?}", prev_mod.identifier.as_ref().unwrap().to_string().as_str());
        log_message!("Adding {} mod items.", prev_mod.mod_items.len());
        log_message!("Module now has {} items.", self.mod_items.len());

        for mut next_mod in prev_mod.mod_items.clone().iter() {
            if prev_mod.identifier.is_none() && self.identifier.is_none() {
                log_message!("was none when parsing {}", self.identifier.clone().unwrap());
                continue;
            } else if prev_mod.identifier.is_none() && self.identifier.is_none() {
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
            let mut attrs;
            if let Item::Mod(ref next_module) = item {
                error!("Found next module: {:?}.", SynHelper::get_str(&next_module));
                attrs = next_module.attrs.clone();
                let mut new_mod = ItemMod {
                    attrs,
                    vis: Visibility::Public(VisPublic{ pub_token: Default::default() }),
                    mod_token: Default::default(),
                    ident,
                    content: Some((Brace::default(), vec![item])),
                    semi: None,
                };
                self.mod_items.push(Item::Mod(new_mod));
            }  else {
                error!("Found item: {:?} was not a module.", SynHelper::get_str(&item));
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

            if dest_path.exists() {
                info!("Writing module to dest path: {:?}", dest_path.to_str().as_ref().unwrap());
                fs::write(dest_path.clone(), existing)
                    .unwrap();
            } else {
                error!("Destination path did not exist when attempting to write.");
            }

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
        info!("Searching for {} with base {}", module_name, base_dir);
        let dirs_with_name = DirectoryWalker::walk_find_mod(module_name, base_dir);

        if dirs_with_name.len() == 1 {
            info!("Found one dirs {:?} when walking.", &dirs_with_name[0].to_str().as_ref().unwrap());
            let correct_buf = dirs_with_name[0].to_owned();
            return Some((correct_buf.clone(), File::open(correct_buf)));
        } else if dirs_with_name.len() == 0 {
            info!("Did not find any dir when walking: {} to find {} from parent {:?}", base_dir, module_name, &parent_buf.to_str().unwrap());
            return None
        } else {
            info!("Found multiple dirs when walking: {:?}", dirs_with_name.iter()
                .flat_map(|d| d.to_str().into_iter().collect::<Vec<_>>())
            );
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
            .map(|c| {
                info!("Found next: {:?}", c.to_str());
                c
            })
    }

}

use std::fs::DirEntry;
use syn::Pat;


use knockoff_logging::*;
use radix_trie::{Trie, TrieCommon};

pub struct DirectoryWalker;

impl DirectoryWalker {

    pub fn walk_find_mod(module_name: &str, base_dir: &str) -> Vec<PathBuf> {
        let mut walk_dir;
        if module_name.ends_with(".rs") {
            walk_dir = Self::walk_dir(module_name.replace(".rs", "").as_str(), base_dir);
        } else {
            walk_dir = Self::walk_dir(module_name, base_dir);
        }
        walk_dir
            .into_iter()
            .map(|f| {
                info!("Searching {:?} for mod", f);
                f
            })
            .collect()
    }

    fn walk_dir(module_name: &str, base_dir: &str) -> Vec<PathBuf> {
        let mut out_bufs = Trie::new();
        Self::find_dir_in_directory(
            &|file| Self::is_mod(file, module_name),
            &|file| true,
            Some(base_dir), &mut out_bufs
        );
        info!("Found {} bufs", out_bufs.len());
        out_bufs.keys().map(|k| Path::new(k).to_path_buf()).collect()
    }

    fn is_mod(path_buf: &PathBuf, mod_name: &str) -> bool {
        if Self::is_parent_mod_name(path_buf, mod_name)
            && Self::file_name_equals(path_buf, "mod.rs") {
            info!("Found mod parent mod name: {}: {:?}", mod_name, path_buf);
            true
        } else {
            if Self::file_name_equals(path_buf, format!("{}.rs", mod_name).as_str()) {
                info!("Found mod file name equals: {}: {:?}", mod_name, path_buf);
                true
            } else {
                false
            }
        }
    }

    fn file_name_matches(path_buf: &PathBuf, to_match: &dyn Fn(&str) -> bool) -> bool {
        if path_buf.file_name().is_none() {
            false
        } else {
            path_buf.file_name().as_ref()
                .filter(|f| f.to_str()
                    .filter(|filename_to_match| to_match(filename_to_match))
                    .is_some()
                )
                .is_some()
        }
    }

    fn file_name_equals(path_buf: &PathBuf, to_match: &str) -> bool {
        if path_buf.file_name().is_none() {
            false
        } else {
            info!("Testing if file name {} is {:?}", to_match, path_buf);
            if path_buf.file_name()
                .filter(|f| f.to_str()
                    .map(|path_buf_file_name| {
                        info!("Testing if {} is same as {}", path_buf_file_name, to_match);
                        path_buf_file_name
                    })
                    .filter(|&parent_dir_name| parent_dir_name == to_match)
                    .is_some()
                )
                .is_some() {
                info!("Found file name {}", to_match);
                true
            } else {
                false
            }
        }
    }

    fn is_parent_mod_name(path: &PathBuf, mod_name: &str) -> bool {
        path.parent().filter(|parent_path| {
            if !parent_path.is_dir() {
                false
            } else {
                info!("Testing if parent {:?} is same as mod file name {:?}", path, mod_name);
                if Self::file_name_equals(&parent_path.to_path_buf(), mod_name) {
                    info!("Found parent file name {:?} is same as mod file name {:?}", path, mod_name);
                    true
                } else {
                    false
                }
            }
        }).is_some()
    }

    pub fn walk_directories_matching_to_path(
        search_file: &dyn Fn(&PathBuf) -> bool,
        search_dir: &dyn Fn(&PathBuf) -> bool,
        base_dir: &str
    ) -> Vec<PathBuf> {
        let mut trie: Trie<String, ()> = Trie::new();
        Self::find_dir_in_directory(search_file,
                                    search_dir,
                                    Some(base_dir), &mut trie);
        trie.keys().map(|p| Path::new(p).to_path_buf()).collect()
    }

    pub fn walk_directories_matching(
        search_file: &dyn Fn(&PathBuf) -> bool,
        search_dir: &dyn Fn(&PathBuf) -> bool,
        base_dir: &str
    ) -> Trie<String, ()> {
        let mut trie: Trie<String, ()> = Trie::new();
        Self::find_dir_in_directory(search_file,
                                    search_dir,
                                    Some(base_dir), &mut trie);
        trie
    }

    pub fn find_dir_in_directory(
        search_add_file: &dyn Fn(&PathBuf) -> bool,
        continue_search_directory: &dyn Fn(&PathBuf) -> bool,
        base_dir_opt: Option<&str>,
        path_bufs: &mut Trie<String, ()>
    ) {
        if base_dir_opt.is_none() {
            return
        }

        let base_dir = base_dir_opt.unwrap();

        OsString::from(base_dir.to_string()).to_str()
            .into_iter()
            .for_each(|os_string_dir| {
                info!("Searching {}", os_string_dir);
                let next_os_path = Path::new(os_string_dir).to_path_buf();
                if search_add_file(&next_os_path) {
                    info!("Inserting value {:?}.", next_os_path);
                    path_bufs.insert(os_string_dir.to_string(), ());
                }

                let dirs = fs::read_dir(next_os_path)
                    .map_err(|e| {
                        error!("Failed to read dirs {:?} when searching for {}", &e, os_string_dir);
                    });

                let _ = dirs.map(|read_dir| {
                    info!("Searching next dir: {:?}", read_dir);
                    let dir_entries = read_dir
                        .filter(|d| d.is_ok())
                        .map(|d| d.unwrap())
                        .collect::<Vec<DirEntry>>();

                    Self::get_dir(search_add_file, continue_search_directory,
                                  dir_entries, path_bufs)
                });
            });
    }

    pub fn get_dir(
        search_add: &dyn Fn(&PathBuf) -> bool,
        continue_search_directory: &dyn Fn(&PathBuf) -> bool,
        read_dir: Vec<DirEntry>,
        path_bufs: &mut Trie<String, ()>
    ) {
        read_dir.iter()
            .for_each(|dir_found| {
                info!("Found next dir entry: {:?}", dir_found);
                let next_dir_path = dir_found.path();
                if search_add(&next_dir_path) {
                    info!("Inserting value.");
                    path_bufs.insert(next_dir_path.to_path_buf().to_str().unwrap().to_string(), ());
                }
                if continue_search_directory(&next_dir_path) {
                    next_dir_path.to_str()
                        .map(|dir_str|
                            Self::find_dir_in_directory(
                                search_add,
                                continue_search_directory,
                                Some(dir_str),
                                path_bufs
                            )
                        );
                }
            });
    }
}
