use module_macro_shared::profile_tree::ProfileTree;
use proc_macro2::TokenStream;
use syn::Item;

pub trait TokenStreamProvider {
    fn new(items: &ProfileTree) -> Self;
    fn generate_token_stream(&self) -> TokenStream;
}

#[test]
fn test() {

}