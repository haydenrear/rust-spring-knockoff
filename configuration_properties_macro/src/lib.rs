use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Item, parse_macro_input};
use codegen_utils::syn_helper::SynHelper;

#[proc_macro_derive(ConfigurationProperties)]
pub fn configuration_properties(ts: TokenStream, attr: TokenStream) -> TokenStream {
    let mut item_found: Item = parse_macro_input!(input_found as Item).into();
    if let Item::Struct(item_struct) = item_found {
        let field_names = item_struct.fields.iter()
            .flat_map(|f|
                f.ident.map(|next_field_ident| SynHelper::get_str(&next_field_ident))
                    .into_iter()
            )
            .collect::<Vec<String>>();
        quote! {
            impl
        }
    }
    TokenStream::default()
}