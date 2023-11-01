use syn::{Field, Generics, ImplItem, ItemImpl, Lifetime, PatType, Type, TypeArray};
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
use enum_fields::EnumFields;
use syn::ext::IdentExt;
use syn::token::Auto;
use codegen_utils::project_directory;
use set_enum_fields::SetEnumFields;
use crate::logger_lazy;
import_logger!("dependency.rs");

#[derive(Clone)]
pub struct DependencyDescriptor {
    pub item_impl: Option<ItemImpl>,
    pub abstract_type: Option<BeanPath>,
    pub profile: Vec<ProfileBuilder>,
    pub path_depth: Vec<String>,
    pub qualifiers: Vec<String>,
    pub item_impl_gens: Generics
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

#[derive(EnumFields, Clone, SetEnumFields)]
pub enum AutowiredType {
    AutowireField {
        qualifier: Option<String>,
        lazy: bool,
        field: Field,
        autowired_type: Type,
        concrete_type_of_field_bean_type: Option<Type>,
        mutable: bool,
        generics: Generics
    }, AutowiredFnArg {
        qualifier: Option<String>,
        profile: Option<String>,
        lazy: bool,
        fn_arg: PatType,
        fn_arg_ident: Ident,
        bean_type: BeanPath,
        autowired_type: Type,
        concrete_type_of_field_bean_type: Option<Type>,
        mutable: bool,
        generics: Generics
    }
}

impl Debug for AutowiredType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AutowiredType::AutowireField{
                field,
                concrete_type_of_field_bean_type,
                ..
            }  => {
                f.write_str("Field: ")?;
                f.write_str(format!("Field type: {:?}, ", SynHelper::get_str(&field)).as_str())?;
                f.write_str(format!("Concrete type: {:?}, ", SynHelper::get_str(&concrete_type_of_field_bean_type)).as_str())?;
            }
            AutowiredType::AutowiredFnArg{
                fn_arg,
                bean_type,
                concrete_type_of_field_bean_type,
                ..
            } => {
                f.write_str(format!("Bean type: {:?}, ", &bean_type).as_str())?;
                f.write_str(format!("Fn arg: {:?}, ", SynHelper::get_str(&fn_arg)).as_str())?;
                f.write_str(format!("Concrete type: {:?}, ", SynHelper::get_str(&concrete_type_of_field_bean_type)).as_str())?;
            }
        }
        f.write_str(format!("Qualifier: {:?}, ", self.qualifier()).as_str())?;
        f.write_str(format!("Profile: {:?}, ", SynHelper::get_str(self.autowired_type())).as_str())?;
        Ok(())
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

#[derive(Clone, EnumFields, SetEnumFields)]
pub enum DependencyMetadata {
    FieldDepType {
        bean_info: AutowiredType,
        lifetime: Option<Lifetime>,
        bean_type: Option<BeanType>,
        array_type: Option<TypeArray>,
        bean_type_path: Option<BeanPath>,
        is_abstract: Option<bool>,
        generics: Generics,
        qualifier: Option<String>
    },
    ArgDepType {
        bean_info: AutowiredType,
        lifetime: Option<Lifetime>,
        bean_type: Option<BeanType>,
        array_type: Option<TypeArray>,
        bean_type_path: Option<BeanPath>,
        is_abstract: Option<bool>,
        generics: Generics,
        qualifier: Option<String>
    }
}


impl DepType for DependencyMetadata {
    fn is_dep_type_abstract(&self) -> bool {
        DependencyMetadata::is_abstract(self)
            .or(Some(false))
            .unwrap()
    }

    fn dep_type_bean_type_path(&self) -> &Option<BeanPath> {
        self.bean_type_path()
    }

    fn dep_type_concrete_type(&self) -> &Option<Type> {
        self.bean_info().concrete_type_of_field_bean_type()
    }

    fn dep_type_maybe_qualifier(&self) -> &Option<String> {
        self.bean_info().qualifier()
    }

    fn dep_type_mutable(&self) -> bool {
        *self.bean_info().mutable()
    }

    fn dep_type_field_ident(&self) -> Ident {
        let i = self.bean_info().field().as_ref()
            .iter().flat_map(|i| i.ident.iter())
            .next()
            .cloned();
        assert!(i.is_some(), "Ident was none!");
        i.unwrap()
    }

    fn dep_type_field_type(&self) -> &Type {
        &self.bean_info().autowired_type()
    }
}

pub trait DepType {
    fn is_dep_type_abstract(&self) -> bool;
    fn dep_type_bean_type_path(&self) -> &Option<BeanPath>;
    fn dep_type_concrete_type(&self) -> &Option<Type>;
    fn dep_type_maybe_qualifier(&self) -> &Option<String>;
    fn dep_type_mutable(&self) -> bool;
    fn dep_type_qualifier(&self) -> String {
        self.dep_type_maybe_qualifier()
            .clone()
            .or(self.dep_type_bean_type_path().as_ref().map(|b| b.get_inner_type_id()))
            .or(Some(self.dep_type_field_type().to_token_stream().to_string().clone()))
            .map(|q| q.to_string())
            .unwrap()
    }
    fn dep_type_identifier(&self) -> String {
        self.dep_type_bean_type_path().as_ref().map(|b| b.get_inner_type_id())
            .or(Some(self.dep_type_field_type().to_token_stream().to_string().clone()))
            .unwrap()
    }
    fn dep_type_field_ident(&self) -> Ident;
    fn dep_type_field_type(&self) -> &Type;
}


