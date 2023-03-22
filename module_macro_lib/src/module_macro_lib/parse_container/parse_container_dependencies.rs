use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::functions::FunctionType;
use module_macro_shared::parse_container::parse_container_builder::BuildParseContainer;
use module_macro_shared::parse_container::ParseContainer;
use crate::module_macro_lib::bean_parser::BeanDependencyParser;

use knockoff_providers_gen::DelegatingParseContainerModifierProvider;

use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::profile_tree::concrete_profile_tree_modifier::ConcreteTypeProfileTreeModifier;
use crate::module_macro_lib::profile_tree::mutable_profile_tree_modifier::MutableProfileTreeModifier;
use crate::module_macro_lib::profile_tree::profile_profile_tree_modifier::ProfileProfileTreeModifier;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::parse_container::parse_container_modifier::ParseContainerModifier;
pub struct DelegateParseContainerModifier;

impl BuildParseContainer for DelegateParseContainerModifier {
    fn build_parse_container(&self, parse_container: &mut ParseContainer) {
        DelegatingParseContainerModifierProvider::do_modify(parse_container);
    }
}

pub struct BuildDependencyParseContainer;

impl BuildParseContainer for BuildDependencyParseContainer {
    fn build_parse_container(&self, parse_container: &mut ParseContainer) {
        Self::add_dependencies_to_bean_definitions(parse_container);
    }
}

impl BuildDependencyParseContainer {
    fn add_dependencies_to_bean_definitions(parse_container: &mut ParseContainer) {
        let keys = parse_container.get_injectable_keys();
        log_message!("{} is the number of injectable keys before.", keys.len());
        for id in keys.iter() {
            let mut removed = parse_container.injectable_types_builder.remove(id).unwrap();
            let deps_set = BeanDependencyParser::add_dependencies(removed, &parse_container.injectable_types_builder, &parse_container.fns);
            parse_container.injectable_types_builder.insert(id.clone().parse().unwrap(), deps_set);
        }
        log_message!("{} is the number of injectable keys after.", parse_container.injectable_types_builder.len());
    }
}

