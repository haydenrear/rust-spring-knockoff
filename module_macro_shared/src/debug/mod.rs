use std::fmt::{Debug, Formatter};
use std::fmt;
use codegen_utils::syn_helper;
use codegen_utils::syn_helper::SynHelper;
use quote::ToTokens;
use crate::aspect::{AspectInfo, MethodAdviceChain};
use crate::bean::{Bean, BeanDefinitionType, BeanPath, BeanPathParts};
use crate::dependency::DepType;

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

impl Debug for BeanPathParts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BeanPathParts::ArcType { arc_inner_types, outer_type  } => {
                let mut debug_struct = f.debug_struct("ArcType");
                debug_struct.field("arc_inner_types", &SynHelper::get_str(arc_inner_types).as_str());
                debug_struct.field("outer_type", &SynHelper::get_str(outer_type).as_str());
                debug_struct.finish()
            }
            BeanPathParts::ArcMutexType { arc_mutex_inner_type, outer_type } => {
                let mut debug_struct = f.debug_struct("ArcMutexType");
                debug_struct.field("arc_mutex_inner_type", &SynHelper::get_str(arc_mutex_inner_type).as_str());
                debug_struct.field("outer_type", &SynHelper::get_str(outer_type).as_str());
                debug_struct.finish()
            }
            BeanPathParts::MutexType { mutex_type_inner_type, outer_type } => {
                let mut debug_struct = f.debug_struct("MutexType");
                debug_struct.field("mutex_type_inner_type", &SynHelper::get_str(mutex_type_inner_type).as_str());
                debug_struct.field("outer_type", &SynHelper::get_str(outer_type).as_str());
                debug_struct.finish()
            }
            BeanPathParts::FnType { input_types, return_type } => {
                let mut debug_struct = f.debug_struct("FnType");
                debug_struct.field("input_types", &SynHelper::get_str(input_types.iter().map(|t| t.to_token_stream().to_string()).collect::<Vec<String>>().join(", ")).as_str());
                debug_struct.field("return_type", &SynHelper::get_str(return_type).as_str());
                debug_struct.finish()
            }
            BeanPathParts::QSelfType { q_self } => {
                let mut debug_struct = f.debug_struct("QSelfType");
                debug_struct.field("q_self", &SynHelper::get_str(q_self).as_str());
                debug_struct.finish()
            }
            BeanPathParts::BindingType { associated_type } => {
                let mut debug_struct = f.debug_struct("BindingType");
                debug_struct.field("associated_type", &SynHelper::get_str(associated_type).as_str());
                debug_struct.finish()
            }
            BeanPathParts::GenType { inner } => {
                let mut debug_struct = f.debug_struct("GenType");
                debug_struct.field("inner", &SynHelper::get_str(inner).as_str());
                debug_struct.finish()
            }
        }
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
