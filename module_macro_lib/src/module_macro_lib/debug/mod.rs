use std::fmt;
use std::fmt::{Debug, DebugStruct, Formatter, Write};
use std::ptr::write;
use proc_macro2::Ident;
use syn::{ItemFn, Type};
use quote::ToTokens;
use module_macro_shared::aspect::{AspectInfo, MethodAdviceChain};
use codegen_utils::syn_helper;
use codegen_utils::syn_helper::{debug_struct_field_opt, debug_struct_field_opt_tokens, SynHelper};
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType, BeanPath, BeanPathParts, BeanType};
use module_macro_shared::dependency::{AutowiredField, DependencyDescriptor, FieldDepType};
use module_macro_shared::functions::FunctionType;
use module_macro_shared::module_tree::Trait;
use crate::module_macro_lib::knockoff_context_builder::aspect_generator::AspectGenerator;
use module_macro_shared::profile_tree::ProfileTree;

impl Debug for AspectGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = f.debug_struct("AspectGenerator");
        f.debug_list()
            .entries(&self.method_advice_aspects);
        Ok(())
    }
}

