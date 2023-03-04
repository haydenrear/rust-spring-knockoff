use syn::__private::{str, TokenStream, ToTokens};
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

    pub fn get_attr_from_vec(autowired_attr: &Vec<Attribute>, matcher_str: Vec<&str>) -> Option<String> {
        autowired_attr.iter()
            .filter(|a| matcher_str.iter().any(|m| Self::get_str(a).as_str().contains(*m)))
            .next()
            .map(|a| SynHelper::parse_attr_path_single(a).or(Some("".to_string())))
            .flatten()
    }

    pub fn get_str<'a, T: ToTokens>(ts: T) -> String {
        ts.to_token_stream().to_string().clone()
    }
}