use syn::Item;
use module_macro_shared::parse_container::ParseContainer;

pub struct SecurityParseProvider;

impl SecurityParseProvider {

    pub fn parse_update(items: &mut Item, parse_container: &mut ParseContainer) {
        match items {
            Item::Mod(http_security_mod) => {
            }
            _ => {

            }
        }
    }
}