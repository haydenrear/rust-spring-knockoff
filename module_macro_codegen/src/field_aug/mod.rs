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
pub struct FieldAug {
    default: Option<TokenStream>
}

impl FieldAug {
    pub(crate) fn new() -> Self {
        Self {
            default: None
        }
    }
}

impl FieldAug {
    fn default_tokens() -> TokenStream {
        let t = quote! {
                #[derive(Parse, Default, Clone, Debug)]
                pub struct FieldAugmenterImpl;

                impl FieldAugmenter for FieldAugmenterImpl {
                    fn process(&self, struct_item: &mut ItemStruct) {
                    }
                }
            }.into();
        t
    }
}

impl CodegenItem for FieldAug {
    fn supports(&self, impl_item: &Item) -> bool {
        match impl_item {
            Item::Fn(impl_item) => {
                impl_item.attrs.iter()
                    .any(|attr_found| attr_found.to_token_stream()
                        .to_string().as_str().contains("field_aug")
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
                    pub struct FieldAugmenterImpl;

                    impl FieldAugmenter for FieldAugmenterImpl {
                        fn process(&self, struct_item: &mut ItemStruct) {
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
        String::from("FieldAug")
    }

    fn default_codegen(&self) -> String {
        FieldAug::default_tokens().to_string()
    }

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        Box::new(self.clone())
    }
}
