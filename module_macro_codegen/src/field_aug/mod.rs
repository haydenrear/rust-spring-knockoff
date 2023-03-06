use std::{env, fs};
use std::any::Any;
use std::fmt::Error;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Block, Item, ItemFn, ItemImpl};
use crate::parser::{CodegenItem, CodegenItemType, LibParser};

#[derive(Clone)]
pub struct FieldAug {
    default: Option<TokenStream>,
    item: Vec<Item>
}

impl FieldAug {

    pub(crate) fn new_dyn_codegen(item: &Vec<Item>) -> Option<CodegenItemType> {
        Self::new(item)
            .map(|i| CodegenItemType::FieldAug(i))
    }

    pub(crate) fn new(item: &Vec<Item>) -> Option<Self> {
        if FieldAug::supports_item(item) {
            return Some(
                Self {
                    default: None,
                    item: item.clone().iter().map(|c| c.clone()).collect()
                }
            );
        }
        None
    }

    fn item_filter(impl_item: &Item) -> bool {
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
}

impl Default for FieldAug {
    fn default() -> Self {
        Self {
            default: None, item: vec![]
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


    fn supports_item(impl_item: &Vec<Item>) -> bool where Self: Sized {
        impl_item.iter().any(|impl_item| {
            Self::item_filter(impl_item)
        })
    }

    fn supports(&self, item: &Vec<Item>) -> bool {
        Self::supports_item(item)
    }

    fn get_codegen(&self) -> Option<String> {

        if self.item.len() == 0 {
            return None;
        }

        let blocks = self.item.iter()
            .filter(|impl_item| Self::item_filter(impl_item))
            .clone()
            .flat_map(|item| {
                match item {
                    Item::Fn(item_fn) => {
                        let block = item_fn.block.deref().clone();
                        vec![block]
                    }
                    _ => {
                        vec![]
                    }
                }
            }).collect::<Vec<Block>>();

        let q = quote! {
            #[derive(Parse, Default, Clone, Debug)]
            pub struct FieldAugmenterImpl;

            impl FieldAugmenter for FieldAugmenterImpl {
                fn process(&self, struct_item: &mut ItemStruct) {
                    #(#blocks)*
                }
            }
        };

        Some(q.to_string())

    }

    fn default_codegen(&self) -> String {
        FieldAug::default_tokens().to_string()
    }

    fn get_unique_id(&self) -> String {
        String::from("FieldAug")
    }
}
