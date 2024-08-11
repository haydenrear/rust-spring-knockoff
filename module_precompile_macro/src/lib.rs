
extern crate proc_macro;

use proc_macro::{TokenStream};
use syn::{Item, parse_macro_input};
use syn::parse::Parser;
use module_precompile_lib::parse_module;

#[proc_macro_attribute]
pub fn boot_knockoff(attr: TokenStream, input: TokenStream) -> TokenStream {
    // this is where import from module_precompile_lib, which imports from generated dfactory, etc.
    // to produce the mutable factories - so then modify the user code for things like adding aspects.
    // unfortunately this means every mutable module needs to be annotated... so then every mutable
    // module also has to be a submodule file.
    let input_found = input.clone();
    parse_module(parse_macro_input!(input_found as Item)).into()
}
