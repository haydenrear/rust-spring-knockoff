use std::{env, fs};
use std::path::Path;
use toml::{Table, Value};
use proc_macro2::{Ident, Span};
use syn::parse_str;
use std::io::Write;
use codegen_utils::env::get_project_dir;
use crate_gen::TomlWriter;
use crate::parse_container_modifier::ParseContainerModifierProvider;
use crate::provider::{DelegatingProvider, Provider, ProviderItem};
use crate::parse_provider::ParseProvider;
use crate::token_provider::TokenProvider;

pub struct FactoriesParser;

macro_rules! providers {
    ($($ty:ident),*) => {

        use proc_macro2::TokenStream;
        use quote::TokenStreamExt;

        impl DelegatingProvider for FactoriesParser {
            fn tokens() -> TokenStream {
                let mut ts = TokenStream::default();
                $(
                    ts.append_all($ty::tokens());
                )*
                ts
            }
            fn deps() -> Vec<ProviderItem> {
                let mut deps = vec![];
                $(
                    let next_deps = $ty::deps();
                    next_deps.iter()
                        .for_each(|dep| {
                            if !deps.contains(dep) {
                                deps.push(dep.clone());
                            }
                        });
                )*
                deps
            }
        }

    }
}

providers!(ParseProvider, TokenProvider, ParseContainerModifierProvider);

impl FactoriesParser {

    pub fn parse_factories(provider_type: &str) -> Provider {

        let knockoff_factories = env::var("KNOCKOFF_FACTORIES")
            .ok()
            .or(Some(get_project_dir("codegen_resources/knockoff_factories.toml")))
            .unwrap();

        let path = Path::new(knockoff_factories.as_str());

        if path.exists() {
            return Provider {
                providers: Self::read_provider_items(path, provider_type)
            };
        }

        Provider {
            providers: vec![],
        }
    }

    fn read_provider_items(path: &Path, provider: &str) -> Vec<ProviderItem> {
        let providers = fs::read_to_string(path)
            .ok()
            .map(|f| {
                f.parse::<Table>().ok()
            })
            .flatten()
            .map(|t| {
                if t.contains_key(provider) {
                    let values = &t[provider];
                    return values.as_table().unwrap().keys()
                        .map(|key| Self::parse_token_provider_item(&values, &key))
                        .collect::<Vec<ProviderItem>>();
                }
                vec![]
            })
            .or(Some(vec![]))
            .unwrap();
        providers
    }

    fn parse_token_provider_item(values: &Value, key: &String) -> ProviderItem {

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

        ProviderItem {
            dep_name: name.clone(),
            name,
            provider_path,
            provider_ident,
            path,
            version,
            deps
        }
    }

    pub fn write_cargo_dependencies(parsed_factories: &Vec<ProviderItem>) -> String {

        let mut cargo_file = vec![];

        parsed_factories.iter()
            .flat_map(|p| {
                p.deps.clone()
            })
            .for_each(|dep| Self::get_dependency(&mut cargo_file, dep));

        String::from_utf8(cargo_file).unwrap()
    }

    pub fn get_dependency(mut cargo_file: &mut Vec<u8>, dep: Dependency) {
        dep.version.map(|version| {
            let features = dep.features.join("\", \"");
            if features.len() != 0 {
                writeln!(&mut cargo_file, "{} = {{ version = \"{}\", features = [\"{}\"] }}", dep.dep_name, version, features)
                    .unwrap();
            } else {
                writeln!(&mut cargo_file, "{} = \"{}\"", dep.dep_name, version).unwrap();
            }
        }).or_else(|| {
            dep.path.map(|path| {
                writeln!(&mut cargo_file, "[dependencies.{}]", dep.dep_name).unwrap();
                writeln!(&mut cargo_file, "path = \"{}\"", path).unwrap();
            })
        });
    }

    fn parse_dependencies(table: &Table) -> Vec<Dependency> {
        if !table.contains_key("dependencies") {
            return vec![];
        }
        let deps = &table["dependencies"];
        deps.as_table().map(|t| {
            t.keys().flat_map(|k| {
                deps[k].as_table()
                    .map(|d| {
                        let features = Self::get_features(d);
                        vec![Dependency {
                            name: k.to_string(),
                            path: Self::get_for_key_or_none(d, "path"),
                            dep_name: k.to_string(),
                            version: Self::get_for_key_or_none(d, "version"),
                            features,
                        }]
                    })
                    .or_else(|| {
                        t[k].as_str()
                            .map(|version| {
                                vec![Dependency {
                                    name: k.to_string(),
                                    path: None,
                                    dep_name: k.to_string(),
                                    version: Some(version.to_string()),
                                    features: vec![],
                                }]
                            })
                    })
                    .or(Some(vec![]))
                    .unwrap()
            })
                .collect::<Vec<Dependency>>()
        })
            .or(Some(vec![]))
            .unwrap()
    }

    fn get_features(d: &Table) -> Vec<String> {
        if !d.contains_key("features") {
            return vec![]
        }
        d["features"].as_array().or(Some(&vec![]))
            .unwrap()
            .iter()
            .flat_map(|v| v.as_str().map(|s| vec![s.to_string()]).or(Some(vec![])).unwrap())
            .collect::<Vec<String>>()
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

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Dependency {
    pub name: String,
    pub path: Option<String>,
    pub dep_name: String,
    pub version: Option<String>,
    pub features: Vec<String>
}
