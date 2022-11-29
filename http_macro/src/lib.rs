#![feature(proc_macro_quote)]
use proc_macro::{quote, Span, TokenStream};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::sync::{Arc, Mutex};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Controller)]
pub fn library(input: TokenStream) -> TokenStream {
    quote! {}
}
