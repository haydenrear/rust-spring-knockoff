
pub mod test;

use std::any::Any;
use std::borrow::BorrowMut;
use std::env;
use std::io::Error;
use std::ops::Deref;
use std::process::id;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, FnArg, Item, ItemFn, Type};
use knockoff_logging::use_logging;
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use module_macro_codegen::parser::{CodegenItem, CodegenItems, LibParser};

use_logging!();

/// There is a way to allow for the user to make changes to the method argument as in method
/// advice in the following steps:
/// 1. Relocate the original method to an aspect function in the same struct called proceed.
///     - You need to take out the function idents of the original function and use those same
///         names for the function args.
pub struct AspectAdviceAdder;



