extern crate proc_macro;

use proc_macro::{TokenStream};
use syn::{Item, parse_macro_input};
use syn::parse::Parser;
use module_macro_lib::module_macro_lib::module_parser::parse_module;

#[proc_macro_attribute]
pub fn module_attr(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_found = input.clone();
    parse_module(parse_macro_input!(input_found as Item)).into()
}