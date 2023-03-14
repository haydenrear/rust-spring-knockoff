use std::{env, fs};
use std::path::Path;
use knockoff_logging::log_message;
use toml::{Table, Value};
use proc_macro2::{Ident, Span};
use syn::parse_str;
use crate::token_provider::{TokenProvider, TokenProviderItem};

pub struct FactoriesParser;

#[test]
fn test_factories_parser() {
    let tp = FactoriesParser::parse_factories();
    assert_eq!(tp.providers.len(), 1);
}

impl FactoriesParser {
    pub(crate) fn parse_factories() -> TokenProvider {

        let knockoff_factories = env::var("KNOCKOFF_FACTORIES")
            .ok()
            .or(Some("/Users/hayde/IdeaProjects/rust-spring-knockoff/codegen_resources/knockoff_factories.toml".to_string()))
            .unwrap();

        let path = Path::new(knockoff_factories.as_str());

        if path.exists() {
            return TokenProvider {
                providers: Self::read_token_provider_items(path)
            };
        }

        TokenProvider {
            providers: vec![],
        }
    }

    fn read_token_provider_items(path: &Path) -> Vec<TokenProviderItem> {
        let providers = fs::read_to_string(path)
            .ok()
            .map(|f| {
                log_message!("Read knockoff factories file with content {}.", &f);
                f.parse::<Table>().ok()
            })
            .flatten()
            .map(|t| {
                let values = &t["token_provider"];
                values.as_table().unwrap().keys()
                    .map(|key| Self::parse_token_provider_item(&values, &key))
                    .collect::<Vec<TokenProviderItem>>()
            })
            .or(Some(vec![]))
            .unwrap();
        providers
    }

    fn parse_token_provider_item(values: &Value, key: &String) -> TokenProviderItem {

        let values = &values[key];
        let provider_path = &values["provider_path"].as_str().unwrap();
        let provider_ident = &values["provider_ident"].as_str().unwrap();
        let provider_ident = Ident::new(provider_ident, Span::call_site());
        let provider_path = parse_str::<syn::Path>(provider_path).unwrap();
        let name = key.as_str().to_string();

        TokenProviderItem {
            name,
            provider_path,
            provider_ident,
        }
    }
}
