use crate::factories_parser::{Dependency, FactoriesParser};
use crate::provider::ProviderItem;

#[test]
fn test_factories_parser() {
    let tp = FactoriesParser::parse_factories("token_provider");
    assert_eq!(tp.providers.len(), 1);
    assert_eq!(tp.providers.iter().flat_map(|t| t.deps.to_owned()).collect::<Vec<Dependency>>().len(), 6);
    let out = FactoriesParser::write_cargo_dependencies(&tp.providers);
    assert!(out.as_str().contains("[dependencies.web_framework]"));
    assert!(out.as_str().contains("path = \"../../web_framework\""));
    assert!(out.as_str().contains("[dependencies.web_framework_shared]"));
    assert!(out.as_str().contains("path = \"../../web_framework_shared\""));
}
