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

#[derive(Clone)]
pub struct Initializer {
    default: Option<TokenStream>
}

impl Initializer {
    pub(crate) fn new() -> Self {
        Self {
            default: None
        }
    }
}

impl Initializer {
    fn default_tokens() -> TokenStream {
        let t = quote! {
                #[derive(Parse, Default, Clone, Debug)]
                pub struct ContextInitializerImpl;

                impl ContextInitializer for ContextInitializerImpl {
                    fn do_update(&self) {
                    }
                }
            }.into();
        t
    }
}

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

    fn default_codegen(&self) -> String {
        Initializer::default_tokens().to_string()
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

    fn get_unique_id(&self) -> String {
        String::from("Initializer")
    }

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        Box::new(self.clone())
    }
}
