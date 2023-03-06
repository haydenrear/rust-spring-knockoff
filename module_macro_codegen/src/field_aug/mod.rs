use std::{env, fs};
use std::any::Any;
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
    default: Option<TokenStream>,
    item: Option<Item>
}

impl FieldAug {

    pub(crate) fn new_dyn_codegen(item: &Item) -> Option<Box<dyn CodegenItem>> {
        Self::new(item)
            .map(|i| Box::new(i) as Box<dyn CodegenItem>)
    }

    pub(crate) fn new_any(item: &Item) -> Option<Box<dyn Any>> {
        Self::new(item)
            .map(|i| Box::new(i) as Box<dyn Any>)
    }

    pub(crate) fn new(item: &Item) -> Option<Self> {
        if FieldAug::supports_item(item) {
            return Some(Self { default: None, item: Some(item.clone()) });
        }
        None
    }

}

impl Default for FieldAug {
    fn default() -> Self {
        Self {
            default: None, item: None
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


    fn supports_item(impl_item: &Item) -> bool where Self: Sized {
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

    fn supports(&self, item: &Item) -> bool {
        Self::supports_item(item)
    }

    fn get_codegen(&self) -> Option<String> {
        self.item.clone().map(|item| {
            match item {
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
        })
            .flatten()
            .or(None)
    }

    fn default_codegen(&self) -> String {
        FieldAug::default_tokens().to_string()
    }

    fn clone_dyn_codegen(&self) -> Box<dyn CodegenItem> {
        Box::new(self.clone())
    }

    fn get_unique_id(&self) -> String {
        String::from("FieldAug")
    }
}
