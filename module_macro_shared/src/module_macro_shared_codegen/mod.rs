use derive_syn_parse::Parse;
use proc_macro2::TokenStream;
use syn::ItemStruct;
use syn::parse::{Parse, ParseStream};

pub trait ContextInitializer: Parse {
    fn do_update(&self);
}

pub trait FieldAugmenter {
    fn process(&self, struct_item: &mut ItemStruct);
}
