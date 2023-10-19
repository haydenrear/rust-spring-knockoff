use std::collections::HashMap;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::Type;
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType, BeanPath, BeanPathParts, BeanType};
use module_macro_shared::parse_container::ParseContainer;
use module_macro_shared::profile_tree::ProfileBuilder;
use crate::module_macro_lib::parse_container::ParseContainerBuilder;
use crate::module_macro_lib::knockoff_context_builder::ApplicationContextGenerator;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;

pub struct ContextBuilder;

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

impl ContextBuilder {

    pub fn build_token_stream(parse_container: &mut ParseContainer) -> TokenStream {
        parse_container.log_app_container_info();
        ParseContainerBuilder::build_parse_container(parse_container);
        let ctx
            = ApplicationContextGenerator::create_context_generator(&mut parse_container.profile_tree);
        ctx.generate_token_stream()
    }

}