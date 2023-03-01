use syn::__private::{str, ToTokens};
use syn::Attribute;

pub mod test;

pub struct SynHelper;

impl SynHelper {
    pub fn parse_attr_path_single(attr: &Attribute) -> Option<String> {
        attr.tokens.to_string().strip_suffix(")")
            .map(|stripped_suffix| stripped_suffix.strip_prefix("("))
            .flatten()
            .map(|stripped| stripped.to_string())
    }
}