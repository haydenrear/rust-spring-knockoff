
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
        Some(
            CodegenItems {
                codegen: LibParser::parse_aspects(),
            }
        )
    }

    pub(crate) fn write_aspect(&self, type_for_aspect: Type, args_for_aspect: Option<Type>, aspect_fn: TokenStream) -> TokenStream {
        if args_for_aspect.is_some() {
            let args = args_for_aspect.unwrap();
            quote! {
                impl AspectWithArgs<#args> for #type_for_aspect {
                    #aspect_fn
                }
            }
        } else {
            quote! {
                impl Aspect> for #type_for_aspect {
                    #aspect_fn
                }
            }
        }
    }

}



