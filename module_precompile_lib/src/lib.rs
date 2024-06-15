use syn::Item;
use proc_macro2::TokenStream;
use quote::ToTokens;
// use ...

// TODO: imported into this code will be Delegating token providers that will have been generated
//       from an entire forward pass over the program (as opposed to only reading a single module,
//       such as authentication gen). During this forward pass, instead of using the Box<dyn Metadata>,
//       it will have generated in-line with a macro for each of them. Then, easily implement as an
//       ItemModifier.
pub fn parse_module(mut item: Item) -> TokenStream {
    item.to_token_stream()
}
