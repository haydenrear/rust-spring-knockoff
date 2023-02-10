use quote::ToTokens;
use syn::Attribute;

pub struct ParseUtil;

impl ParseUtil {
    pub(crate) fn filter_singleton_prototype(attr: &Vec<Attribute>) -> Option<&Attribute> {
        attr.into_iter()
            .filter(|&attr| {
                let attr_name = attr.to_token_stream().to_string();
                println!("Checking attribute: {} for fn.", attr_name.clone());
                attr_name.contains("singleton") || attr_name.contains("prototype")
            }).next()
    }

    pub fn strip_value(value: &str) -> Option<String> {
        println!("Stripping prefix {}.", value);
        value.strip_prefix("#[singleton(")
            .map(|without_singleton| {
                println!("{} is without singleton", without_singleton);
                without_singleton.strip_prefix("#[prototype(")
                    .or(Some(without_singleton)).unwrap()
            })
            .map(|without_prefix| {
                println!("{} is without singleton and prototype", without_prefix);
                without_prefix.strip_suffix(")]")
                    .map(|str| String::from(str))
                    .or(None)
            }).unwrap_or(None).map(|value_found| {
            println!("Found bean with qualifier {}.", value_found.as_str());
            value_found
        })
    }

    pub fn strip_value_attr(attr: &Attribute) -> Option<String> {
        Self::strip_value(attr.to_token_stream().to_string().as_str())
    }


}