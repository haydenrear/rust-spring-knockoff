use std::{env, fs};
use std::path::Path;
use toml::{Table, Value};
use proc_macro2::{Ident, Span};
use syn::parse_str;
use crate::token_provider::{TokenProvider, TokenProviderItem};
use std::io::Write;

pub struct FactoriesParser;

impl FactoriesParser {
    pub fn parse_factories() -> TokenProvider {

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
        let path = values["path"].as_str().map(|s| s.to_string());
        let provider_ident = &values["provider_ident"].as_str().unwrap();
        let version = Self::get_version(values);
        let provider_ident = Ident::new(provider_ident, Span::call_site());
        let provider_path = parse_str::<syn::Path>(provider_path).unwrap();
        let name = key.as_str().to_string();
        let deps = values.as_table().map(|t| Self::parse_dependencies(t))
            .or(Some(vec![]))
            .unwrap();

        TokenProviderItem {
            dep_name: name.clone(),
            name,
            provider_path,
            provider_ident,
            path,
            version,
            deps
        }
    }

    pub fn write_cargo_dependencies(parsed_factories: &TokenProvider) -> String {
        let mut cargo_file = vec![];
        parsed_factories.providers.iter()
            .flat_map(|p| {
                p.deps.clone()
            })
            .for_each(|dep| Self::write_dependency(&mut cargo_file, dep));
        String::from_utf8(cargo_file).unwrap()
    }

    fn write_dependency(mut cargo_file: &mut Vec<u8>, dep: Dependency) {
        dep.version.map(|version| {
            writeln!(&mut cargo_file, "{} = \"{}\"", dep.dep_name, version).unwrap();
        }).or_else(|| {
            dep.path.map(|path| {
                writeln!(&mut cargo_file, "[dependencies.{}]", dep.dep_name).unwrap();
                writeln!(&mut cargo_file, "path = \"{}\"", path).unwrap();
            })
        });
    }

    fn parse_dependencies(table: &Table) -> Vec<Dependency> {
        let deps = &table["dependencies"];
        deps.as_table().map(|t| t.keys().flat_map(|k| {
            deps[k].as_table()
                .map(|t| {
                    vec![Dependency {
                        name: k.to_string(),
                        path: Self::get_for_key_or_none(t, "path"),
                        dep_name: k.to_string(),
                        version: Self::get_for_key_or_none(t, "version")
                    }]
                })
                .or(Some(vec![]))
                .unwrap()
        }).collect::<Vec<Dependency>>()).or(Some(vec![])).unwrap()
    }

    fn get_for_key_or_none(t: &Table, x: &str) -> Option<String> {
        if t.contains_key(x) {
            return t[x].as_str().map(|p| p.to_string());
        }
        None
    }

    fn get_version(values: &Value) -> Option<String> {
        if values.as_table()
            .filter(|t| t.keys().map(|k| k.as_str())
                .collect::<Vec<&str>>()
                .contains(&"version")
            )
            .is_some() {
            return values["version"].as_str().map(|s| s.to_string())
        }
        None
    }
}

#[derive(Clone)]
pub struct Dependency {
    pub name: String,
    pub path: Option<String>,
    pub dep_name: String,
    pub version: Option<String>
}
