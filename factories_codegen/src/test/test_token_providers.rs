use std::path::Path;
use codegen_utils::project_directory;
use crate::factories_parser::{FactoriesParser};
use crate::provider::{DelegatingProvider, ProviderProvider};
use crate::token_provider::TokenProvider;


#[test]
fn test_parse_factories() {
    let out = FactoriesParser::parse_factories_value();
    assert!(out.is_some());
    let parse_provider = out.as_ref().unwrap().parse_provider.as_ref().unwrap();
    let security_parse_provider = parse_provider.get("security_parse_provider");
    let security_parse_provider_provider_data = &security_parse_provider.unwrap().provider_data;
    assert!(security_parse_provider_provider_data.is_some());
    let security_parse_provider_dependency_data = &security_parse_provider.unwrap().dependency_data;
    assert!(security_parse_provider_dependency_data.is_some());
    let handler_mapping = out.as_ref().unwrap().token_provider.as_ref().unwrap().get("handler_mapping").unwrap();
    let handler_mapping_data = &handler_mapping.provider_data;
    assert!(handler_mapping_data.is_some());

    let handler_mapping_dependency_data = &handler_mapping.dependency_data;
    assert!(handler_mapping_dependency_data.is_some());

    let security_parse_provider_path = &security_parse_provider_provider_data.as_ref().unwrap().provider_path;
    assert!(security_parse_provider_path.is_some());
    let handler_mapping_data = &handler_mapping_data.as_ref().unwrap().provider_ident;
    assert!(handler_mapping_data.is_some());

    let deps = &out.as_ref().unwrap().dependencies;
    assert!(deps.is_some());
}

#[test]
fn test_token_providers() {
    let factories = FactoriesParser::parse_factories_value().unwrap();
    assert_eq!(factories.token_provider.as_ref().unwrap().len(), 1);
    let token_provider = factories.token_provider;
    let token_provider = token_provider.as_ref().unwrap();
    let option = token_provider.get("handler_mapping");
    let next = option.as_ref().unwrap();
    assert_eq!(next.provider_data.as_ref().unwrap().provider_ident.as_ref().unwrap(), "HandlerMappingTokenProvider");
    assert_eq!(next.provider_data.as_ref().unwrap().provider_path.as_ref().unwrap(), "handler_mapping::HandlerMappingBuilder");
    let token_provider = TokenProvider::create_token_provider(next);
    println!("{}", &token_provider.to_string());
    assert_ne!(token_provider.to_string().replace(" ", "").len(), 0);
}
