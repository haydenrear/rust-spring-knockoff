use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{FnArg, Type};
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::BeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;

use crate::module_macro_lib::knockoff_context_builder::SynHelper;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("factory_factory_generator.rs");

pub struct FactoryBeanBeanFactoryGenerator {
    concrete_bean_factories: Vec<BeanFactoryInfo>,
    abstract_bean_factories: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for FactoryBeanBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

impl BeanFactoryGenerator for FactoryBeanBeanFactoryGenerator {

    fn get_concrete_factories(&self) -> Vec<BeanFactoryInfo> {
        self.concrete_bean_factories.clone()
    }

    fn get_abstract_factories(&self) -> Vec<BeanFactoryInfo> {
        self.abstract_bean_factories.clone()
    }

    fn new(concrete_bean_factories: Vec<BeanFactoryInfo>, abstract_bean_factories: Vec<BeanFactoryInfo>) -> Self {
        Self {
            concrete_bean_factories,
            abstract_bean_factories,
        }
    }
}
