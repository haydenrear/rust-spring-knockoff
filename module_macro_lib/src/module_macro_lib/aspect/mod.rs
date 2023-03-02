
pub mod test;

use std::any::Any;
use std::env;
use std::io::Error;
use std::ops::Deref;
use std::process::id;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Attribute, FnArg, Item, ItemFn, Type};
use knockoff_logging::{use_logging};
use module_macro_codegen::aspect::MethodAdviceAspect;
use module_macro_codegen::parser::{CodegenItem, CodegenItems, LibParser};

use_logging!();

#[derive(Default, Clone)]
pub struct AspectParser {
    pub aspects: Vec<MethodAdviceAspect>
}

impl AspectParser {
    pub(crate) fn parse_aspects() -> Option<CodegenItems> {
        let codegen = env::var("AUG_FILE").map(|aug_file| {
                LibParser::parse_codegen_items(&aug_file)
                    .iter().filter(|c| c.get_unique_id().as_str().contains("MethodAdviceAspect"))
                    .map(|b| b.clone_dyn_codegen())
                    .collect::<Vec<Box<dyn CodegenItem>>>()
            }).or(Ok::<Vec<Box<dyn CodegenItem>>, Error>(vec![])).unwrap();
        Some(
            CodegenItems{
                codegen,
            }
        )
    }
}



