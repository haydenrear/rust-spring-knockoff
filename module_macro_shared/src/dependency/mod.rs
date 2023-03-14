use syn::{Field, ItemImpl, Lifetime, Type, TypeArray};
use std::fmt::{Debug, Formatter};
use std::fmt;
use codegen_utils::syn_helper;
use std::cmp::Ordering;
use quote::ToTokens;
use crate::bean::{BeanPath, BeanType};
use crate::profile_tree::ProfileBuilder;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::logging::executor;
use crate::logging::StandardLoggingFacade;

impl Eq for AutowireType {}

impl PartialEq<Self> for AutowireType {
    fn eq(&self, other: &Self) -> bool {
        self.item_impl.to_token_stream().to_string().eq(&other.item_impl.to_token_stream().to_string())
    }
}

impl PartialOrd<Self> for AutowireType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.item_impl.to_token_stream().to_string().partial_cmp(&other.item_impl.to_token_stream().to_string())
    }
}

impl Ord for AutowireType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.item_impl.to_token_stream().to_string().cmp(&other.item_impl.to_token_stream().to_string())
    }
}

#[derive(Clone)]
pub struct AutowireType {
    pub item_impl: ItemImpl,
    pub profile: Vec<ProfileBuilder>,
    pub path_depth: Vec<String>,
    pub qualifiers: Vec<String>
}

#[derive(Clone)]
pub struct AutowiredField {
    pub qualifier: Option<String>,
    pub profile: Option<String>,
    pub lazy: bool,
    pub field: Field,
    pub type_of_field: Type,
    pub concrete_type_of_field_bean_type: Option<Type>,
    pub mutable: bool
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

#[derive(Clone)]
pub struct DepType {
    pub bean_info: AutowiredField,
    pub lifetime: Option<Lifetime>,
    pub bean_type: Option<BeanType>,
    pub array_type: Option<TypeArray>,
    pub bean_type_path: Option<BeanPath>,
    pub is_abstract: Option<bool>
}
