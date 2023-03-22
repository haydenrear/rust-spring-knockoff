use crate::factories_parser::{Dependency, FactoriesParser};
use crate::provider::ProviderItem;
use crate::provider::DelegatingProvider;

#[test]
fn test_factories_parser() {
    let tp = <FactoriesParser as DelegatingProvider>::deps();
    assert_eq!(tp.len(), 2);
    let out = FactoriesParser::write_cargo_dependencies(&tp);
    assert_eq!(tp.iter().flat_map(|t| t.deps.to_owned()).collect::<Vec<Dependency>>().len(), 6);
    assert!(out.as_str().contains("[dependencies.web_framework]"));
    assert!(out.as_str().contains("path = \"../../web_framework\""));
    assert!(out.as_str().contains("[dependencies.web_framework_shared]"));
    assert!(out.as_str().contains("path = \"../../web_framework_shared\""));
    println!("{} is out", out);
}
