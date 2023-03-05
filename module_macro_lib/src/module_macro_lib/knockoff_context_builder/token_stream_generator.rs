use proc_macro2::TokenStream;

pub trait TokenStreamGenerator {
    fn generate_token_stream(&self) -> TokenStream;
}