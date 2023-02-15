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


pub struct FieldAug;
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
}
