use std::env;
use quote::ToTokens;
use syn::Item;
use crate::aspect::{AspectParser, MethodAdviceAspectCodegen};
use crate::parser::{CodegenItem, LibParser};

pub mod test_aspect;
