use std::fmt::{Debug, Formatter};
use std::fmt;
use std::ops::Deref;
use codegen_utils::syn_helper;
use codegen_utils::syn_helper::SynHelper;
use quote::ToTokens;
use crate::bean::{BeanDefinition, BeanDefinitionType, BeanPath, BeanPathParts};
use crate::dependency::DependencyMetadata;

impl Debug for BeanPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("BeanPath").unwrap();
        f.debug_list()
            .entries(&self.path_segments)
            .finish()
    }
}

impl Debug for BeanDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("Bean");
        syn_helper::debug_struct_field_opt(&mut debug_struct, &self.ident.as_ref(), "bean ident");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.struct_type.as_ref(), "struct type");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.struct_found.as_ref(), "item struct");
        syn_helper::debug_struct_vec_field_tokens("fields", &mut debug_struct, &self.fields.as_ref());
        syn_helper::debug_struct_vec_field_debug("profile", &mut debug_struct, &self.profile);
        syn_helper::debug_struct_vec_field_debug("traits_impl", &mut debug_struct, &self.traits_impl);
        syn_helper::debug_struct_vec_field_debug("path_dep", &mut debug_struct, &self.path_depth);
        syn_helper::debug_struct_vec_field_debug("deps_map", &mut debug_struct, &self.deps_map);
        Ok(())
    }
}

impl Debug for DependencyMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("DepType");
        syn_helper::debug_debug_struct_field_opt(&mut debug_struct, &self.bean_type(), "bean_type");
        debug_struct.field("bean_info", &self.bean_info());
        syn_helper::debug_debug_struct_field_opt(&mut debug_struct, &self.bean_type_path(), "bean_type_path");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.lifetime(), "lifetime");
        syn_helper::debug_struct_field_opt_tokens(&mut debug_struct, &self.array_type(), "array_type");
        debug_struct.finish()
    }
}




impl Debug for BeanPathParts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BeanPathParts::ArcType { inner_ty: arc_inner_types, outer_path: outer_type } => {
                let mut debug_struct = f.debug_struct("ArcType");
                debug_struct.field("arc_inner_types", &SynHelper::get_str(arc_inner_types).as_str());
                debug_struct.field("outer_type", &SynHelper::get_str(outer_type).as_str());
                debug_struct.finish()
            }
            BeanPathParts::ArcMutexType { inner_ty: arc_mutex_inner_type, outer_path: outer_type } => {
                let mut debug_struct = f.debug_struct("ArcMutexType");
                debug_struct.field("arc_mutex_inner_type", &SynHelper::get_str(arc_mutex_inner_type).as_str());
                debug_struct.field("outer_type", &SynHelper::get_str(outer_type).as_str());
                debug_struct.finish()
            }
            BeanPathParts::MutexType { inner_ty: mutex_type_inner_type, outer_path: outer_type } => {
                let mut debug_struct = f.debug_struct("MutexType");
                debug_struct.field("mutex_type_inner_type", &SynHelper::get_str(mutex_type_inner_type).as_str());
                debug_struct.field("outer_type", &SynHelper::get_str(outer_type).as_str());
                debug_struct.finish()
            }
            BeanPathParts::FnType { inner_tys: input_types, return_type_opt: return_type } => {
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
            BeanPathParts::GenType { gen_type , inner_ty_opt: inner, ..} => {
                let mut debug_struct = f.debug_struct("GenType");
                debug_struct.field("gen_type", &SynHelper::get_str(gen_type).as_str());
                if let Some(inner_ty) = inner {
                    debug_struct.field("inner", &SynHelper::get_str(inner_ty).as_str());
                }
                debug_struct.finish()
            }
            BeanPathParts::BoxType { inner_ty: inner } => {
                let mut debug_struct = f.debug_struct("BoxType");
                debug_struct.field("inner", &SynHelper::get_str(inner).as_str());
                debug_struct.finish()
            }
            BeanPathParts::PhantomType { inner_bean_path_parts: inner } => {
                let mut debug_struct = f.debug_struct("PhantomData");
                debug_struct.field("inner", &inner);
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
