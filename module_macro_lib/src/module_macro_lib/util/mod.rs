use quote::ToTokens;
use syn::Attribute;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub struct ParseUtil;

impl ParseUtil {
    pub(crate) fn filter_att<'a>(attr: &'a Vec<Attribute>, attr_names: Vec<&'a str>) -> Option<&'a Attribute> {
        attr.into_iter()
            .filter(|&attr| {
                let attr_name = attr.to_token_stream().to_string();
                log_message!("Checking attribute: {} for fn.", attr_name.clone());
                attr_names.iter()
                    .any(|attr_name_to_check|
                        attr_name.contains(attr_name_to_check)
                    )
            }).next()
    }

    pub fn strip_value(value: &str, attribute_prefix: Vec<&str>) -> Option<String> {
        log_message!("Stripping prefix {}.", value);
        let mut value = value;
        for attr_prefix in attribute_prefix.iter() {
            value = value.strip_prefix(attr_prefix)
                .or(Some(value)).unwrap();
        }
        value = value.strip_suffix(")]")
            .or(Some(value)).unwrap();

        Some(String::from(value))
    }

    pub fn strip_value_attr(attr: &Attribute, prefix: Vec<&str>) -> Option<String> {
        Self::strip_value(attr.to_token_stream().to_string().as_str(), prefix)
    }

}