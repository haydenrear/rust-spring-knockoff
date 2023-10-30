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
use syn::token::Brace;
use build_lib::replace_modules;
use codegen_utils::env::{get_project_base_build_dir, get_build_project_dir};
use module_macro_codegen::parser::LibParser;

use knockoff_logging::*;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/authentication_gen_build.log"));

fn main() {
    log_message!("Initializing module macro lib.");
    let aug_file = get_aug_file();
    log_message!("Found augmented file: {}.", aug_file.as_str());
    LibParser::do_codegen(&aug_file, "codegen.rs");
    let mut cargo_change = "cargo:rerun-if-changed=".to_string();
    cargo_change += get_project_base_build_dir().as_str();
    println!("{}", cargo_change);
}

fn get_aug_file() -> String {
    let aug_file = env::var("AUG_FILE").ok()
        .or(Some(String::from(get_build_project_dir("codegen_resources/knockoff_test_aug.rs"))))
        .unwrap();
    aug_file
}