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

fn main() {
    replace_modules(
        Some("/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test/src"),
        vec![".git/HEAD"]
    );
    println!()
}

fn create_log_file() -> File {
    let mut log_file = File::create(
        Path::new("/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test/src")
            .join("log.txt")
    ).unwrap();
    log_file
}