use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::fmt::{Debug, Formatter, Pointer};
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::sync::{Arc, Mutex};
use syn::{Attribute, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, ImplItem, ImplItemMethod, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Path, QSelf, TraitItem, Type, TypeArray, TypePath};
use syn::__private::str;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{
    Ident,
    LitStr,
    Token,
    token::Paren,
};
use quote::{format_ident, IdentFragment, quote, quote_token, TokenStreamExt, ToTokens};
use syn::Data::Struct;
use syn::token::{Bang, For, Token};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use crate::module_macro_lib::parse_container::ParseContainer;

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

#[derive(Clone)]
pub struct Bean {
    pub struct_type: Option<Type>,
    pub struct_found: Option<ItemStruct>,
    pub traits_impl: Vec<AutowireType>,
    pub enum_found: Option<ItemEnum>,
    pub attr: Vec<Attribute>,
    // A reference to another DepImpl - the id is the Type.
    pub deps_map: Vec<DepType>,
    pub id: String,
    pub path_depth: Vec<String>,
    pub profile: Vec<Profile>,
    pub ident: Option<Ident>,
    pub fields: Vec<Fields>,
    pub bean_type: Option<BeanType>,
    pub mutable: bool
}

#[derive(Clone)]
pub enum BeanDefinitionType {
    Abstract {
        bean: Bean,
        dep_type: AutowireType
    }, Concrete {
        bean: Bean,
    }
}

impl Debug for BeanDefinitionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BeanDefinitionType::Abstract { bean, dep_type } => {
                log_message!("{} is bean id.", bean.id.as_str());
            }
            BeanDefinitionType::Concrete { bean } => {
                log_message!("{} is bean id.", bean.id.as_str());
            }
        };
        Ok(())
    }
}

#[derive(Clone)]
pub struct BeanPath {
    pub(crate) path_segments: Vec<BeanPathParts>
}

impl BeanPathParts {
    pub fn is_mutable(&self) -> bool {
        match self {
            BeanPathParts::ArcType { .. } => {
                false
            }
            BeanPathParts::ArcMutexType { arc_mutex_inner_type, outer_type } => {
                log_message!("Found mutex for {}, returning true!", SynHelper::get_str(arc_mutex_inner_type.clone()));
                log_message!("Found mutex, returning true!");
                true
            }
            BeanPathParts::MutexType { mutex_type_inner_type, outer_type } => {
                log_message!("Found mutex for {}, returning true!", SynHelper::get_str(mutex_type_inner_type.clone()));
                true
            }
            BeanPathParts::FnType { .. } => {
                false
            }
            BeanPathParts::QSelfType { .. } => {
                false
            }
            BeanPathParts::BindingType { .. } => {
                false
            }
            BeanPathParts::GenType { .. } => {
                false
            }
        }
    }
}

impl BeanPath {
    pub fn get_autowirable_type(&self) -> Option<Type> {
        log_message!("Found {} path segments.", self.path_segments.len());
        self.path_segments.iter().for_each(|f| {
            log_message!("{:?} is the bean path part", f.clone());
        });
        match &self.path_segments.clone()[..] {
            [ BeanPathParts::MutexType{ mutex_type_inner_type, outer_type}, BeanPathParts::GenType {inner} ] => {
                log_message!("Found mutex type with type {} and gen type {}.", SynHelper::get_str(mutex_type_inner_type.clone()), SynHelper::get_str(inner.clone()));
                Some(inner.clone())
            }
            [  BeanPathParts::ArcType{ arc_inner_types , outer_type}, BeanPathParts::GenType { inner }] => {
                log_message!("Found arc type with type {} and gen type {}.", SynHelper::get_str(arc_inner_types.clone()).as_str(), SynHelper::get_str(inner.clone()));
                Some(inner.clone())
            }
            [ BeanPathParts::ArcMutexType{ arc_mutex_inner_type, outer_type: out}, BeanPathParts::MutexType { mutex_type_inner_type, outer_type} ] => {
                log_message!("Found arc mutex type with type {} and gen type {}.", SynHelper::get_str(arc_mutex_inner_type.clone()).as_str(), SynHelper::get_str(mutex_type_inner_type.clone()));
                Some(mutex_type_inner_type.clone())
            }
            [ BeanPathParts::FnType { input_types, return_type }  ] => {
                log_message!("Found fn and mutex type with type {}.", SynHelper::get_str(return_type.clone()).as_str());
                return_type.clone()
            }
            [ BeanPathParts::QSelfType { q_self } ] => {
                log_message!("Found qself type with type {}.", SynHelper::get_str(q_self.clone()).as_str());
                Some(q_self.clone())
            }
            [ BeanPathParts::BindingType { associated_type } ] => {
                log_message!("Found binding type with type {}.", SynHelper::get_str(associated_type.clone()).as_str());
                Some(associated_type.clone())
            }
            [ BeanPathParts::GenType { inner } ] => {
                log_message!("Found gen type with type {}.", SynHelper::get_str(inner.clone()).as_str());
                Some(inner.clone())
            }
            [..] => {
                None
            }
        }
    }
}

#[derive(Clone)]
pub enum BeanPathParts {
    ArcType {
        arc_inner_types: Type,
        outer_type: Path
    },
    ArcMutexType {
        arc_mutex_inner_type: Type,
        outer_type: Path
    },
    MutexType {
        mutex_type_inner_type: Type,
        outer_type: Path
    },
    //TODO: add recursion for return type to create additional bean path part
    FnType {
        input_types: Vec<Type>,
        return_type: Option<Type>
    },
    QSelfType {
        q_self: Type
    },
    BindingType {
        associated_type: Type
    },
    GenType {
        inner: Type
    }
}

impl Debug for BeanPathParts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BeanPathParts::ArcType { arc_inner_types, .. } => {
                log_message!("Found arc inner type: {}.", SynHelper::get_str(arc_inner_types).as_str());
            }
            BeanPathParts::ArcMutexType { arc_mutex_inner_type, .. } => {
                log_message!("Found arc mutex type: {}.", SynHelper::get_str(arc_mutex_inner_type).as_str());
            }
            BeanPathParts::MutexType { mutex_type_inner_type, .. } => {
                log_message!("Found mutex type: {}.", SynHelper::get_str(mutex_type_inner_type).as_str());
            }
            BeanPathParts::FnType { input_types, return_type } => {
                log_message!("Found return type.");
            }
            BeanPathParts::QSelfType { q_self } => {
                log_message!("Found q self type: {}.", SynHelper::get_str(q_self).as_str());
            }
            BeanPathParts::BindingType { associated_type } => {
                log_message!("Found associated type: {}.", SynHelper::get_str(associated_type).as_str());
            }
            BeanPathParts::GenType { inner } => {
                log_message!("Found gen type: {}.", SynHelper::get_str(inner).as_str());
            }
        }
        Ok(())
    }
}

/**
Will be annotated with #[bean] and #[singleton], #[prototype] as provided factory functions.
 **/
pub struct ModulesFunctions {
    pub fn_found: FunctionType
}

#[derive(Clone)]
pub enum FunctionType {
    Singleton(ItemFn, Option<String>, Option<Type>),
    Prototype(ItemFn, Option<String>, Option<Type>)
}

#[derive(Clone)]
pub struct AutowireType {
    pub item_impl: ItemImpl,
    pub profile: Vec<Profile>,
    pub path_depth: Vec<String>
}

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash, Debug)]
pub struct Profile {
    pub(crate) profile: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            profile: "DefaultProfile".to_string()
        }
    }
}

#[derive(Clone)]
pub struct DepType {
    pub bean_info: AutowiredField,
    pub lifetime: Option<Lifetime>,
    pub bean_type: Option<BeanType>,
    pub array_type: Option<TypeArray>,
    pub bean_type_path: Option<BeanPath>
}

#[derive(Clone, Debug)]
pub enum BeanType {
    // contains the identifier and the qualifier as string
    Singleton(BeanDefinition, Option<FunctionType>),
    Prototype(BeanDefinition, Option<FunctionType>)
}


#[derive(Clone)]
pub struct BeanDefinition {
    pub qualifier: Option<String>,
    pub bean_type_type: Option<Type>,
    pub bean_type_ident: Option<Ident>,
}

#[derive(Clone)]
pub struct AutowiredField {
    pub qualifier: Option<String>,
    pub lazy: bool,
    pub field: Field,
    pub type_of_field: Type,
    pub mutable: bool
}

impl Default for Bean {
    fn default() -> Self {
        Self {
            struct_type: None,
            struct_found: None,
            traits_impl: vec![],
            attr: vec![],
            enum_found: None,
            deps_map: vec![],
            id: String::default(),
            path_depth: vec![],
            profile: vec![],
            ident: None,
            fields: vec![],
            bean_type: None,
            mutable: false
        }
    }
}

pub struct Trait {
    pub trait_type: Option<ItemTrait>,
}

impl Trait {
    pub fn new(trait_type: ItemTrait) -> Self {
        Self {
            trait_type: Some(trait_type)
        }
    }
}

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct InjectableTypeKey {
    pub underlying_type: String,
    pub impl_type: Option<String>,
    pub profile: Vec<Profile>
}

pub enum GetBeanResultError {
    BeanNotInContext, BeanDependenciesNotSatisfied
}


