use proc_macro2::TokenStream;
use crate::profile_tree::ProfileTree;

pub trait ProfileTreeTokenProvider {
    fn new(items: &mut ProfileTree) -> Self;
    fn generate_token_stream(&self) -> TokenStream;
}

pub trait ProfileTreeFrameworkTokenProvider {
    fn new(items: &mut ProfileTree) -> Self;
    fn generate_token_stream(&self) -> TokenStream;
}
