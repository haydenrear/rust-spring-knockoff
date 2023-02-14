use std::{env, fs};
use std::fmt::Error;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Item, ItemFn, ItemImpl};

pub fn write_initializer(in_dir_file: &str, log_file: &mut File) {
    let in_path = Path::new(in_dir_file);
    if in_path.exists() {
        fs::read_to_string(in_path).ok().and_then(|mut in_file_result| {
            log_file.write("in file exists".as_bytes()).unwrap();
            syn::parse_file(in_file_result.as_str()).ok().or(None)
        })
        .map(|parsed_content| {
            log_file.write("found parsed content".as_bytes()).unwrap();
            get_initializer_impl(&parsed_content, log_file)
        })
            .flatten()
        .map(|impl_item_found| {
            log_file.write("found impl item".as_bytes()).unwrap();
            let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
            File::create(out_path).map(|mut out_file_create| {
                out_file_create.write(get_impl_codegen(impl_item_found, log_file).to_string().as_bytes())
                    .unwrap();
            }).or_else(|err| {
                Ok::<(), Error>(())
            }).unwrap();
        });
    }
}

fn get_impl_codegen(item_fn: ItemFn, log_file: &mut File) -> TokenStream {
    let block = item_fn.block.deref().clone();

    log_file.write("writing itemfn".as_bytes()).unwrap();
    quote! {
            use derive_syn_parse::Parse;
            use module_macro_shared::module_macro_shared_codegen::{ContextInitializer, FieldAugmenter};
            use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Item, ItemMod, ItemStruct, FieldsNamed, FieldsUnnamed, ItemImpl, ImplItem, ImplItemMethod, parse_quote, parse, Type, ItemTrait, Attribute, ItemFn, Path, TraitItem, Lifetime, TypePath, QSelf, TypeArray, ItemEnum};
            use syn::parse::Parser;
            use quote::{quote, ToTokens};

            #[derive(Parse, Default, Clone, Debug)]
            pub struct ContextInitializerImpl;
            #[derive(Parse, Default, Clone, Debug)]
            pub struct FieldAugmenterImpl;

            impl ContextInitializer for ContextInitializerImpl {
                fn do_update(&self) {
                    #block
                }
            }

            // TODO:
            impl FieldAugmenter for FieldAugmenterImpl {
                fn process(&self, struct_item: &mut ItemStruct) {
                    match &mut struct_item.fields {
                        Fields::Named(ref mut fields_named) => {
                            fields_named.named.push(
                                Field::parse_named.parse2(quote!(
                                    pub a: String
                                ).into()).unwrap()
                            )
                        }
                        Fields::Unnamed(ref mut fields_unnamed) => {}
                        _ => {}
                    }
                }
            }

    }
}

fn get_initializer_impl(parsed_content: &syn::File, log_file: &mut File) -> Option<ItemFn> {
    for parsed_item in &parsed_content.items {
        log_file.write("checking item with name: ".as_bytes()).unwrap();
        log_file.write(parsed_item.to_token_stream().to_string().as_bytes()).unwrap();
        match parsed_item {
            Item::Fn(impl_item) => {
                if impl_item.attrs.iter().any(|attr_found| attr_found.to_token_stream().to_string().as_str().contains("initializer")) {
                    log_file.write("Found initializer on item".as_bytes()).unwrap();
                    log_file.write(parsed_item.to_token_stream().to_string().as_bytes()).unwrap();
                    return Some(impl_item.clone());
                }
            }
            other => {
            }
        }
    }
    None
}