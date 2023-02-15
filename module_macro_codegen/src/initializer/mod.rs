use std::{env, fs};
use std::fmt::Error;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Item, ItemFn, ItemImpl};
use crate::parser::{CodegenItem, LibParser};

pub struct Initializer;
impl CodegenItem for Initializer {
    fn supports(&self, impl_item: &Item) -> bool {
        match impl_item {
            Item::Fn(impl_item) => {
                impl_item.attrs.iter()
                    .any(|attr_found| attr_found.to_token_stream()
                        .to_string().as_str().contains("initializer")
                    )
            }
            _ => {
                false
            }
        }
    }

    fn get_codegen(&self, item_fn: &Item) -> Option<String> {
        match item_fn {
            Item::Fn(item_fn) => {

                let block = item_fn.block.deref().clone();

                let q = quote! {

                    #[derive(Parse, Default, Clone, Debug)]
                    pub struct ContextInitializerImpl;

                    impl ContextInitializer for ContextInitializerImpl {
                        fn do_update(&self) {
                            #block
                        }
                    }
                };
                Some(q.to_string())
            }
            _ => {
                None
            }
        }
    }
}
