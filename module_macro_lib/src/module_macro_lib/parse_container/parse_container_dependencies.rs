use module_macro_shared::functions::FunctionType;
use module_macro_shared::parse_container::BuildParseContainer;
use module_macro_shared::parse_container::ParseContainer;

use knockoff_providers_gen::DelegatingParseContainerModifierProvider;

use crate::module_macro_lib::profile_tree::concrete_profile_tree_modifier::ConcreteTypeProfileTreeModifier;
use crate::module_macro_lib::profile_tree::mutable_profile_tree_modifier::MutableProfileTreeModifier;
use crate::module_macro_lib::profile_tree::profile_profile_tree_modifier::ProfileProfileTreeModifier;
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::parse_container::ParseContainerModifier;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use module_macro_shared::BeanDependencyParser;
use crate::logger_lazy;
import_logger!("parse_container_dependency.rs");

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
            info!("Removing {:?} from injectable types builder.", id);
            let mut removed = parse_container.injectable_types_builder.remove(id).unwrap();
            let deps_set = BeanDependencyParser::add_dependencies(removed, &parse_container.injectable_types_builder, &parse_container.fns);
            info!("Adding {:?} to injectable types builder.", id);
            parse_container.injectable_types_builder.insert(id.clone().parse().unwrap(), deps_set);
        }
        log_message!("{} is the number of injectable keys after.", parse_container.injectable_types_builder.len());
    }
}

