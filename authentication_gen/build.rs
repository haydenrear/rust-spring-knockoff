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
use codegen_utils::env::get_project_base_dir;
use codegen_utils::project_directory;
use module_macro_codegen::parser::LibParser;

use knockoff_logging::{initialize_log, initialize_logger, use_logging, create_logger_expr};

use_logging!();
initialize_logger!(TextFileLoggerImpl, StandardLogData, concat!(project_directory!(), "log_out/authentication_gen.log"));
initialize_log!();

fn main() {
    log_message!("Initializing module macro lib.");
    let aug_file = get_aug_file();
    log_message!("Found augmented file: {}.", aug_file.as_str());
    LibParser::do_codegen(&aug_file, "codegen.rs");
    let mut cargo_change = "cargo:rerun-if-changed=".to_string();
    cargo_change += get_project_base_dir().as_str();
    println!("{}", cargo_change);
}

fn get_aug_file() -> String {
    let aug_file = env::var("AUG_FILE").ok()
        .or(Some(String::from("~/IdeaProject/rust-spring-knockoff/codegen_resources/knockoff_test_aug.rs")))
        .unwrap();
    aug_file
}