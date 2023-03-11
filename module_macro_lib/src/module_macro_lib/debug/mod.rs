use std::fmt;
use std::fmt::{Debug, DebugStruct, Formatter, Write};
use std::ptr::write;
use proc_macro2::Ident;
use syn::{ItemFn, Type};
use quote::ToTokens;
use codegen_utils::syn_helper;
use codegen_utils::syn_helper::{debug_struct_field_opt, debug_struct_field_opt_tokens};
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
use crate::module_macro_lib::knockoff_context_builder::aspect_generator::AspectGenerator;
use crate::module_macro_lib::module_tree::{AspectInfo, AutowiredField, AutowireType, Bean, BeanDefinition, BeanDefinitionType, BeanPath, BeanType, DepType, FunctionType, MethodAdviceChain, Trait};
use crate::module_macro_lib::profile_tree::ProfileTree;

impl Debug for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let debug_struct = &mut f.debug_struct("FunctionType");
        syn_helper::debug_struct_field_opt(debug_struct, &self.qualifier, "qualifier");
        syn_helper::debug_struct_field_opt_tokens(debug_struct, &self.fn_type, "singleton type");
        debug_struct.field("item_fn", &self.item_fn.to_token_stream().to_string().as_str());
        debug_struct.field("bean_type", &self.bean_type);
        Ok(())
    }
}

impl Debug for BeanPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("BeanPath").unwrap();
        f.debug_list()
            .entries(&self.path_segments)
            .finish()
    }
}

impl Debug for Bean {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("Bean");
        syn_helper::debug_struct_field_opt(&mut debug_struct, &self.ident.as_ref(), "bean ident");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.struct_type.as_ref(), "struct type");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.struct_found.as_ref(), "item struct");
        syn_helper::debug_struct_vec_field_tokens("fields", &mut debug_struct, &self.fields.as_ref());
        syn_helper::debug_struct_vec_field_debug("profile", &mut debug_struct, &self.profile);
        syn_helper::debug_struct_vec_field_debug("traits_impl", &mut debug_struct, &self.traits_impl);
        syn_helper::debug_struct_vec_field_debug("attrs", &mut debug_struct, &self.attr.iter().map(|t| t.to_token_stream().to_string().clone()).collect::<Vec<String>>());
        syn_helper::debug_struct_vec_field_debug("aspect_info", &mut debug_struct, &self.aspect_info);
        syn_helper::debug_struct_vec_field_debug("path_dep", &mut debug_struct, &self.path_depth);
        syn_helper::debug_struct_vec_field_debug("deps_map", &mut debug_struct, &self.deps_map);
        Ok(())
    }
}


impl Debug for DepType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("DepType");
        syn_helper::debug_debug_struct_field_opt(&mut debug_struct, &self.bean_type, "bean_type");
        debug_struct.field("bean_info", &self.bean_info);
        syn_helper::debug_debug_struct_field_opt(&mut debug_struct, &self.bean_type_path, "bean_type_path");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.lifetime, "lifetime");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.array_type, "array_type");
        debug_struct.finish()
    }
}

impl Debug for AutowiredField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct1 = f.debug_struct("AutowiredField");
        let mut debug_struct = debug_struct1
            .field("mutable", &self.mutable)
            .field("lazy", &self.lazy)
            .field("field", &self.field.to_token_stream().to_string().as_str());
        syn_helper::debug_struct_field_opt(&mut debug_struct, &self.qualifier, "qualifier");
        debug_struct.finish()
    }
}

impl Debug for MethodAdviceChain {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("MethoAdviceChain");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.before_advice, "before_advice");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.after_advice, "after_advice");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.proceed_statement, "proceed_statement");
        debug_struct.finish()
    }
}

impl Debug for AspectInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("AspectInfo");
        syn_helper::debug_struct_vec_field_debug("advice chain", &mut debug_struct, &self.advice_chain);
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.return_type, "return_type");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.method, "method_before");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.method_after, "method_after");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.original_fn_logic, "original_fn_logic");
        debug_struct.field("method_advice_aspect", &self.method_advice_aspect);
        debug_struct.field("mutable", &self.mutable.to_string().as_str());
        debug_struct.field("args", &self.args.iter().map(|a| {
            let mut type_and_ident = "Ident: ".to_string();
            type_and_ident +=  a.0.to_string().as_str();
            type_and_ident += "Type: ";
            type_and_ident += a.0.to_string().as_str();
            type_and_ident
        }).collect::<Vec<String>>().join(", ").as_str());
        debug_struct.finish()
    }
}

impl Debug for AutowireType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let profiles = self.profile.iter().map(|p| p.profile.clone()).collect::<Vec<String>>().join(", ");
        f.debug_struct("AutowireType")
            .field("profiles", &profiles)
            .field("path_depth", &self.path_depth.join(".").as_str())
            .field("item_impl", &self.item_impl.to_token_stream().to_string().as_str())
            .finish()
    }
}

impl Debug for BeanDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("BeanDefinition");
        debug_struct_field_opt(&mut debug_struct, &self.qualifier, "qualifier");
        debug_struct_field_opt_tokens(&mut debug_struct,  &self.bean_type_ident, "bean_type_ident");
        debug_struct_field_opt_tokens(&mut debug_struct,  &self.bean_type_type, "bean_type_type");
        debug_struct.finish()
    }
}

impl Debug for BeanDefinitionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            BeanDefinitionType::Abstract { bean, dep_type } => {
                let mut debug_struct = f.debug_struct("BeanDefinitionType::Abstract");
                debug_struct.field("bean", bean);
                debug_struct.field("dep_type", dep_type);
            }
            BeanDefinitionType::Concrete { bean } => {
                let mut debug_struct = f.debug_struct("BeanDefinitionType::Concrete");
                debug_struct.field("bean", bean);
            }
        }
        Ok(())
    }
}

impl Debug for AspectGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = f.debug_struct("AspectGenerator");
        f.debug_list()
            .entries(&self.method_advice_aspects);
        Ok(())
    }
}

