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
use module_macro_codegen::parser::LibParser;

fn main() {
    let file = &mut create_log_file();
    file.write("initializing...".as_bytes()).unwrap();
    let aug_file = env::var("AUG_FILE").ok()
        .or(Some(String::from("~/IdeaProject/rust-spring-knockoff/module_macro_lib/resources/default_aug.rs")))
        .unwrap();
    file.write("Found aug file: ".as_bytes()).unwrap();
    file.write(aug_file.as_bytes()).unwrap();
    file.write("Found another".as_bytes()).unwrap();
    LibParser::do_codegen(&aug_file, file, false, "codegen.rs");
    print!("cargo:rerun-if-changed=.git/HEAD");
}

fn create_log_file() -> File {
    let mut log_file = File::create(
        Path::new("/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test/src")
            .join("module.log")
    ).unwrap();
    log_file
}