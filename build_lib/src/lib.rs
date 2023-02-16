use std::{env, fs};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::Path;
use std::ptr::write;
use syn::__private::{Span, ToTokens};
use syn::{braced, Fields, Ident, Item, ItemMod, ItemStruct, Token, token, Visibility, VisPublic};
use syn::__private::quote::__private::push_div_eq_spanned;
use syn::parse::{ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::token::Brace;
use knockoff_logging::{initialize_log, initialize_logger, use_logging};

use_logging!();
initialize_logger!(TextFileLogger, StandardLogData);
initialize_log!();

pub fn replace_modules(base_env: Option<&str>, rerun_files: Vec<&str>) {

    let mut file_result = File::open(
        Path::new(base_env.unwrap())
                .join("lib.rs")
    )
        .or_else(|f| {
            log_info!("Failed to open lib.rs".to_string(), "1".to_string());
            let err = f.to_string();
            log_info!(err, "1".to_string());
            Err(())
        });

    if file_result.is_err() {
        return;
    }

    let mut file = file_result.unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("spring-knockoff.rs");
    if File::open(dest_path.clone()).is_ok() {
        fs::remove_file(&dest_path.clone())
            .unwrap();
    }

    File::create(&dest_path)
        .unwrap();

    let lib_file = parse_syn_file(&mut file);

    for mut x in lib_file.items {
        let created_mod = parse_macro(&mut x);
        if created_mod.is_some() {
            let mod_created = created_mod.unwrap();
            log_message!("{}", "here is the finished lib file:".to_string());
            let message = mod_created.to_token_stream().to_string();
            log_message!("{}", message);
            let mut existing = fs::read_to_string(dest_path.clone())
                .unwrap();
            existing.push_str(mod_created.to_token_stream().to_string().as_str());
            fs::write(dest_path.clone(), existing)
                .unwrap();
        }
    }

    rerun_files.iter().for_each(|rerun_file| {
        print!("cargo:rerun-if-changed={}", rerun_file);
    })

}

fn parse_syn_file(file: &mut File) -> syn::File {
    let mut src = String::new();
    file.read_to_string(&mut src)
        .unwrap();
    let mut lib_file = syn::parse_file(&src)
        .unwrap();
    lib_file
}

fn parse_macro(x: &mut Item) -> Option<&mut ItemMod> {
    match x {
        Item::Mod(ref mut module) => {
            let found_inner = module.content.clone().unwrap();

            let mut make_change_bool = false;
            let mut cfg_attr = 0;
            let mut counter = 0;

            for attr in module.attrs.clone().iter() {
                if attr.to_token_stream().to_string().as_str().contains("module_attr") {
                    write_to_log("found attr on main module");
                    write_to_log(attr.tokens.to_string().as_str());
                    write_to_log("Found with module_attr");
                    make_change_bool = true;
                } else if attr.to_token_stream().to_string().as_str().contains("cfg")
                    && attr.to_token_stream().to_string().as_str().contains("springknockoff") {
                    cfg_attr = counter;
                }
                counter += 1;
            }

            if make_change_bool {
                make_change(module, &found_inner, module.ident.to_string().as_str());
                if cfg_attr != 0 {
                    module.attrs.remove(cfg_attr);
                }
                return Some(module);
            }
            None
        }
        _ => {
            None
        }
    }
}

fn make_change(
    module: &mut ItemMod,
    found_inner: &(Brace, Vec<Item>),
    outer_module_name: &str
) {
    let mut counter = 0;
    for item in &found_inner.1 {
        let option = inner_macro(item);
        let module_span = module.span().clone();
        if option.is_some() {
            match &mut module.content {
                None => {
                    write_to_log("Did not find inner macro");
                }
                Some(ref mut item) => {
                    write_to_log("the inner module name is ");
                    write_to_log(module.ident.to_string().as_str());
                    write_to_log("the module module name is ");
                    write_to_log(outer_module_name);
                    let mod_to_replace = get_module_to_replace(
                        option.unwrap().as_str(), outer_module_name, module_span
                    );
                    match mod_to_replace {
                        None => {}
                        Some(mod_found) => {
                            write_to_log("replacing item mod named");
                            write_to_log(mod_found.ident.to_token_stream().to_string().as_str());
                            let mut mod_to_return = mod_found.clone();
                            mod_to_return.vis = Visibility::Public(VisPublic { pub_token: Default::default() });
                            let mut item_mod_created = Item::Mod(mod_to_return.clone());
                            std::mem::replace(item.1.get_mut(counter).unwrap(), item_mod_created);
                            counter += 1;
                        }
                    }
                }
            }
        } else {
        }
    }
}

fn get_module_to_replace(module_name: &str, base: &str, span: Span) -> Option<ItemMod> {
    let mut module_rs_file = String::from(module_name);
    module_rs_file.push_str(".rs");
    let inner_module_path = Path::new(OsString::from("/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test/src").deref())
        .join(base)
        .join(module_rs_file.clone());
    let mut inner_module_file = File::open(inner_module_path);
    let mut new_mod = ItemMod {
        attrs: vec![],
        vis: Visibility::Inherited,
        mod_token: Default::default(),
        ident: Ident::new(module_name, span.clone()),
        content: None,
        semi: None,
    };
    let brace = token::Brace { span: span.clone() };
    let mut items = vec![];
    if inner_module_file.is_ok() {
        let syn_found = parse_syn_file(&mut inner_module_file.unwrap());
        for item in syn_found.items {
            items.push(item.clone());
            write_to_log("parsed inner file and found");
            write_to_log(item.to_token_stream().to_string().as_str());
        }
    } else {
        write_to_log("Did not find");
        write_to_log(module_rs_file.as_str());
    }
    new_mod.content = Some((brace, items));
    Some(new_mod)
}

fn inner_macro(item: &Item) -> Option<String> {
    match &item {
        Item::Mod(module) => {
            write_to_log("found inner module");
            write_to_log(module.ident.to_string().as_str());
            Some(module.ident.to_token_stream().to_string())
        }
        _ => {
            None
        }
    }

}


fn write_to_log(to_write: &str) {
    let to_write = to_write.to_string();
    log_message!("{}", to_write);
}

pub trait NewComponent<T> {
    fn new() -> T;
}