use proc_macro2::TokenStream;
use syn::Item;

pub trait MutableModuleModifier {

    fn matches(item: &mut Item) -> bool;

    fn do_provide(item: &mut Item) -> Option<TokenStream>;

}