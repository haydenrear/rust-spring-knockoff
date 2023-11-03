use std::env;
use quote::ToTokens;
use syn::Item;
use crate::parser::{CodegenItem, LibParser};

#[cfg(test)]
pub mod test_aspect;
#[cfg(test)]
pub mod test_write_multiple;