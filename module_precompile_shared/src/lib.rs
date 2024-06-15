
pub mod module_precompile_shared {
    pub mod module_precompile_tokens {
        use proc_macro2::TokenStream;
        use syn::Item;

        pub trait PrecompileTokenProvider {

            fn matches_item(&self, item: Item) -> bool;

            fn get_replacement(&self, item: Item) -> TokenStream;

        }
    }



}

pub use module_precompile_shared::*;