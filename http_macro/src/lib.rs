#![feature(proc_macro_quote)]
use proc_macro::{quote, Span, TokenStream};
use std::collections::{HashMap, LinkedList};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use syn::{DeriveInput, parse_macro_input};
use std::sync::{Arc, Mutex};
use delegator_macro_rules::types;

// macro_rules! controller {
//     ($ident:ident) => {
//         $ident
//     }
// }