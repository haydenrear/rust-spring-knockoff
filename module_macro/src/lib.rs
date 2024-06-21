extern crate proc_macro;

use proc_macro::{TokenStream};
use std::env;
use syn::{Item, parse_macro_input};
use syn::parse::Parser;
use codegen_utils::program_src;
use module_macro_lib::module_macro_lib::parse_module::parse_module;
use module_macro_shared::parse_module_into_container;

#[proc_macro_attribute]
pub fn module_attr(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_found = input.clone();
    let parent_dir = env::var("PROGRAM_SRC_PATH_FROM_ROOT").map(|s| program_src!(&s))
        .ok()
        .or(Some(program_src!("src")))
        .unwrap();
    parse_module(parse_macro_input!(input_found as Item), &parent_dir).into()
}