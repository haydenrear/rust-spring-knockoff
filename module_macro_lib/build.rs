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
    let mut log_file = File::create(
        Path::new("/Users/hayde/IdeaProjects/rust-spring-knockoff/module_macro_lib/src")
                .join("log.txt")
    ).unwrap();
    replace_modules(
        Some("/Users/hayde/IdeaProjects/rust-spring-knockoff/module_macro_lib/src"),
        &mut log_file,
        vec![".git/HEAD"]
    );
}