use syn::{Field, ImplItem, ItemImpl, Lifetime, PatType, Type, TypeArray};
use std::fmt::{Debug, Formatter};
use std::fmt;
use codegen_utils::syn_helper;
use std::cmp::Ordering;
use proc_macro2::Ident;
use quote::ToTokens;
use codegen_utils::syn_helper::SynHelper;
use crate::bean::{BeanPath, BeanType};
use crate::profile_tree::ProfileBuilder;

use crate::functions::{FunctionType, ModulesFunctions};
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("dependency.rs");

#[derive(Clone)]
pub struct DependencyDescriptor {
    pub item_impl: Option<ItemImpl>,
    pub abstract_type: Option<BeanPath>,
    pub profile: Vec<ProfileBuilder>,
    pub path_depth: Vec<String>,
    pub qualifiers: Vec<String>
}

impl DependencyDescriptor {
    pub fn has_fn_named(&self, name: &str) -> bool {
        self.item_impl.as_ref().filter(|i| {
            i.items.iter().any(|t| {
                match t {
                    ImplItem::Const(_) => false,
                    ImplItem::Method(m) => {
                        let method_name = format!("\"{}\"", name);
                        let method_name = method_name.as_str();
                        info!("Testing if {:?} is same as {}", SynHelper::get_str(&m.sig.ident), method_name);
                        let has_same = m.sig.ident.to_token_stream().to_string().as_str() == name;
                        info!("{} is result to test.", has_same);
                        has_same
                    }
                    ImplItem::Type(_) => false,
                    ImplItem::Macro(_) => false,
                    ImplItem::Verbatim(_) => false,
                    _ => false
                }
            })
        }).is_some()
    }
}
impl PartialEq<Self> for DependencyDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.item_impl.as_ref().map(|i| SynHelper::get_str(&i))
            .eq(&other.item_impl.as_ref().map(|i| SynHelper::get_str(&i)))
            && self.abstract_type.eq(&other.abstract_type)
    }
}

impl Eq for DependencyDescriptor {}

#[derive(Clone)]
pub struct AutowiredField {
    pub qualifier: Option<String>,
    pub lazy: bool,
    pub field: Field,
    pub autowired_type: Type,
    pub concrete_type_of_field_bean_type: Option<Type>,
    pub mutable: bool
}

#[derive(Clone)]
pub struct AutowiredFnArg {
    pub qualifier: Option<String>,
    pub profile: Option<String>,
    pub lazy: bool,
    pub fn_arg: PatType,
    pub fn_arg_ident: Ident,
    pub bean_type: BeanPath,
    pub autowired_type: Type,
    pub concrete_type_of_field_bean_type: Option<Type>,
    pub mutable: bool
}

pub enum AutowiredType {
    Field(AutowiredField), FnArg(AutowiredFnArg)
}

macro_rules! get_ref {
    ($($name:ident, $ty:ty),*) => {
        $(
            impl AutowiredType {
                pub fn $name<'a>(&'a self) -> &'a $ty {
                    match self {
                        AutowiredType::Field(field) => {
                            &field.$name
                        }
                        AutowiredType::FnArg(fn_arg) => {
                            &fn_arg.$name
                        }
                    }
                }
            }
        )*
    }
}

get_ref!(
    autowired_type, Type,
    qualifier, Option<String>
);

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

impl Debug for AutowiredFnArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug_struct1 = f.debug_struct("AutowiredFnArg");
        let mut debug_struct = debug_struct1
            .field("mutable", &self.mutable)
            .field("lazy", &self.lazy)
            .field("field", &self.fn_arg.to_token_stream().to_string().as_str());
        syn_helper::debug_struct_field_opt(&mut debug_struct, &self.qualifier, "qualifier");
        debug_struct.finish()
    }
}

impl Debug for DependencyDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let profiles = self.profile.iter().map(|p| p.profile.clone()).collect::<Vec<String>>().join(", ");
        f.debug_struct("AutowireType")
            .field("profiles", &profiles)
            .field("path_depth", &self.path_depth.join(".").as_str())
            .field("item_impl", &self.item_impl.to_token_stream().to_string().as_str())
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum DependencyMetadata {
    FieldDepType(FieldDepType),
    ArgDepType(ArgDepType)
}

impl DependencyMetadata {

    pub fn get<T>(&self, getter: &dyn Fn(&dyn DepType) -> T) -> T {
        match self {
            DependencyMetadata::FieldDepType(dep_type) => {
               getter(dep_type)
            }
            DependencyMetadata::ArgDepType(dep_type) => {
                getter(dep_type)
            }
        }
    }

    pub fn get_ref<'a, T>(&'a self, getter: &'a dyn Fn(&'a dyn DepType) -> &'a T) -> &'a T {
        match self {
            DependencyMetadata::FieldDepType(dep_type) => {
                getter(dep_type)
            }
            DependencyMetadata::ArgDepType(dep_type) => {
                getter(dep_type)
            }
        }
    }

    pub fn qualifier(&self) -> String {
        self.get(&|d| d.qualifier())
    }

    pub fn is_abstract(&self) -> bool {
        self.get(&|d| d.is_abstract())
    }

    pub fn bean_type_path(&self) -> &Option<BeanPath> {
        self.get_ref(&|d| d.bean_type_path())
    }

    pub fn concrete_type(&self) -> &Option<Type> {
        self.get_ref(&|d| d.concrete_type())
    }

    pub fn field_ident(&self) -> Ident {
        self.get(&|d| d.field_ident())
    }

    pub fn mutable(&self) -> bool {
        self.get(&|d| d.mutable())
    }


    pub fn maybe_qualifier(& self) -> &Option<String> {
        self.get_ref(&|d| d.maybe_qualifier())
    }


    pub fn type_path(&self) -> &Option<BeanPath> {
        self.get_ref(&|d| d.bean_type_path())
    }

    pub fn identifier(&self) -> String {
        self.get(&|d| d.identifier())
    }

    pub fn set_mutable(&mut self) {
        match self {
            DependencyMetadata::FieldDepType(dep_type) => {
                dep_type.bean_info.mutable = true;
            }
            DependencyMetadata::ArgDepType(dep_type) => {
                dep_type.bean_info.mutable = true;
            }
        }
    }

    pub fn set_is_abstract(&mut self) {
        match self {
            DependencyMetadata::FieldDepType(dep_type) => {
                dep_type.is_abstract = Some(true);
            }
            DependencyMetadata::ArgDepType(dep_type) => {
                dep_type.is_abstract = Some(true);
            }
        }
    }

    pub fn set_concrete_field_type(&mut self, concrete_field_type: Type) {
        match self {
            DependencyMetadata::FieldDepType(dep_type) => {
                dep_type.bean_info.concrete_type_of_field_bean_type = Some(concrete_field_type);
            }
            DependencyMetadata::ArgDepType(dep_type) => {
                dep_type.bean_info.concrete_type_of_field_bean_type = Some(concrete_field_type);
            }
        }
    }

}

pub trait DepType {
    fn is_abstract(&self) -> bool;
    fn bean_type_path(&self) -> &Option<BeanPath>;
    fn concrete_type(&self) -> &Option<Type>;
    fn maybe_qualifier(&self) -> &Option<String>;
    fn mutable(&self) -> bool;
    fn qualifier(&self) -> String {
        self.maybe_qualifier()
            .clone()
            .or(self.bean_type_path().as_ref().map(|b| b.get_inner_type_id()))
            .or(Some(self.field_type().to_token_stream().to_string().clone()))
            .map(|q| q.to_string())
            .unwrap()
    }
    fn identifier(&self) -> String {
        self.bean_type_path().as_ref().map(|b| b.get_inner_type_id())
            .or(Some(self.field_type().to_token_stream().to_string().clone()))
            .unwrap()
    }
    fn field_ident(&self) -> Ident;
    fn field_type(&self) -> &Type;
}

impl DepType for FieldDepType {
    fn is_abstract(&self) -> bool {
        self.is_abstract.is_some() && self.is_abstract.unwrap()
    }

    fn bean_type_path(&self) -> &Option<BeanPath> {
        &self.bean_type_path
    }

    fn concrete_type(&self) -> &Option<Type> {
        &self.bean_info.concrete_type_of_field_bean_type
    }
    fn maybe_qualifier(&self) -> &Option<String> {
        &self.bean_info.qualifier
    }

    fn mutable(&self) -> bool {
        self.bean_info.mutable
    }

    fn field_ident(&self) -> Ident {
        assert!(self.bean_info.field.ident.is_some(), "Field ident was None.");
        self.bean_info.field.ident.as_ref().unwrap().clone()
    }

    fn field_type(&self) -> &Type {
        &self.bean_info.autowired_type
    }
}
impl DepType for ArgDepType {
    fn is_abstract(&self) -> bool {
        self.is_abstract.is_some() && self.is_abstract.unwrap()
    }

    fn bean_type_path(&self) -> &Option<BeanPath> {
        &self.bean_type_path
    }

    fn concrete_type(&self) -> &Option<Type> {
        &self.bean_info.concrete_type_of_field_bean_type
    }

    fn maybe_qualifier(&self) -> &Option<String> {
        &self.bean_info.qualifier
    }

    fn mutable(&self) -> bool {
        self.bean_info.mutable
    }

    fn field_ident(&self) -> Ident {
        self.bean_info.fn_arg_ident.clone()
    }

    fn field_type(&self) -> &Type {
        &self.bean_info.autowired_type
    }
}

#[derive(Clone)]
pub struct FieldDepType {
    pub bean_info: AutowiredField,
    pub lifetime: Option<Lifetime>,
    pub bean_type: Option<BeanType>,
    pub array_type: Option<TypeArray>,
    pub bean_type_path: Option<BeanPath>,
    pub is_abstract: Option<bool>,
}

#[derive(Clone)]
pub struct ArgDepType {
    pub bean_info: AutowiredFnArg,
    pub lifetime: Option<Lifetime>,
    pub bean_type: Option<BeanType>,
    pub array_type: Option<TypeArray>,
    pub bean_type_path: Option<BeanPath>,
    pub is_abstract: Option<bool>,
}
