use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};

pub trait ContextInitializer: Parse {
    fn do_update(&self);
}

pub struct ContextInitializerImpl{

}
