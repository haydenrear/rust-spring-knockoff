use syn::{Attribute, Field, Fields, ItemEnum, ItemImpl, ItemStruct, Lifetime, Path, Type, TypeArray};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::fmt;
use codegen_utils::syn_helper;
use quote::ToTokens;
use proc_macro2::Ident;
use syn::__private::str;
use codegen_utils::syn_helper::SynHelper;
use crate::aspect::AspectInfo;
use crate::dependency::{AutowireType, DepType};

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::logging::executor;
use crate::logging::StandardLoggingFacade;
use crate::profile_tree::ProfileBuilder;

#[derive(Clone, Debug)]
pub enum BeanType {
    // contains the identifier and the qualifier as string
    Singleton, Prototype
}

impl BeanType {
    fn new(name: &str) -> Self {
        if name.to_lowercase() == "singleton" {
            return BeanType::Singleton;
        }
        BeanType::Prototype
    }
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

#[derive(Clone)]
pub struct BeanPath {
    pub path_segments: Vec<BeanPathParts>
}

impl BeanPath {

    pub fn is_mutable(&self) -> bool {
        self.path_segments.iter().any(|p| p.is_mutable())
    }

    pub fn is_not_mutable(&self) -> bool {
        self.path_segments.iter().all(|p| !p.is_mutable())
    }

}


impl BeanPath {

    pub fn get_autowirable_type(&self) -> Option<Type> {
        log_message!("Found {} path segments.", self.path_segments.len());
        self.path_segments.iter().for_each(|f| {
            log_message!("{:?} is the bean path part", f.clone());
        });
        let return_type = self.get_inner_type();
        if return_type.is_none() {
            log_message!("Did not find inner type for path segments.");
        } else {
            log_message!("Found inner type: {}.", SynHelper::get_str(return_type.as_ref().unwrap()));
        }
        return_type
    }

    fn are_types_equal(ty: &Type, inner: &Type) -> bool {
        inner.to_token_stream().to_string().as_str() == ty.to_token_stream().to_string().as_str()
    }

    pub fn bean_path_part_matches(&self, ty: &Type) -> bool {
        self.get_inner_type_id() == ty.to_token_stream().to_string()
    }

    pub fn get_inner_type_id(&self) -> String {
        log_message!("Found {} path segments.", self.path_segments.len());
        self.path_segments.iter().for_each(|f| {
            log_message!("{:?} is the bean path part", f.clone());
        });
        self.get_inner_type()
            .map(|t| t.to_token_stream().to_string())
            .or(Some("".to_string()))
            .unwrap()
    }

    pub fn get_inner_type(&self) -> Option<Type> {
        log_message!("Found {} path segments for {:?}.", self.path_segments.len(), &self);
        // TODO: Fix this - not matching for Box (dyn Mutable)
        match &self.path_segments.clone()[..] {
            [ BeanPathParts::ArcMutexType{ arc_mutex_inner_type, outer_type: out},
              BeanPathParts::MutexType { mutex_type_inner_type, outer_type},
              BeanPathParts::GenType { inner } ] => {
                log_message!("Found arc mutex type with type {} and gen type {}.", SynHelper::get_str(arc_mutex_inner_type.clone()).as_str(), SynHelper::get_str(mutex_type_inner_type.clone()));
                return Some(inner.clone());
            }
            [ BeanPathParts::MutexType{ mutex_type_inner_type, outer_type}, BeanPathParts::GenType {inner} ] => {
                log_message!("Found mutex type with type {} and gen type {}.", SynHelper::get_str(mutex_type_inner_type.clone()), SynHelper::get_str(inner.clone()));
                return Some(inner.clone());
            }
            [  BeanPathParts::ArcType{ arc_inner_types , outer_type}, BeanPathParts::GenType { inner }] => {
                log_message!("Found arc type with type {} and gen type {}.", SynHelper::get_str(arc_inner_types.clone()).as_str(), SynHelper::get_str(inner.clone()));
                return Some(inner.clone());
            }
            [ BeanPathParts::ArcMutexType{ arc_mutex_inner_type, outer_type: out}, BeanPathParts::MutexType { mutex_type_inner_type, outer_type} ] => {
                log_message!("Found arc mutex type with type {} and gen type {}.", SynHelper::get_str(arc_mutex_inner_type.clone()).as_str(), SynHelper::get_str(mutex_type_inner_type.clone()));
                return Some(mutex_type_inner_type.clone());
            }
            [ BeanPathParts::MutexType { mutex_type_inner_type, outer_type }  ] => {
                log_message!("Found fn and mutex type with type {}.", SynHelper::get_str(mutex_type_inner_type).as_str());
                return Some(mutex_type_inner_type.clone());
            }
            [ BeanPathParts::FnType { input_types, return_type }  ] => {
                log_message!("Found fn and mutex type with type {}.", SynHelper::get_str(return_type.clone()).as_str());
                return return_type.clone();
            }
            [ BeanPathParts::QSelfType { q_self } ] => {
                log_message!("Found qself type with type {}.", SynHelper::get_str(q_self.clone()).as_str());
                return Some(q_self.clone());
            }
            [ BeanPathParts::BindingType { associated_type } ] => {
                log_message!("Found binding type with type {}.", SynHelper::get_str(associated_type.clone()).as_str());
                return None;
            }
            [  BeanPathParts::ArcType{ arc_inner_types , outer_type} ] => {
                log_message!("Found arc type with type {}.", SynHelper::get_str(arc_inner_types.clone()).as_str());
                return Some(arc_inner_types.clone());
            }
            [ BeanPathParts::GenType { inner } ] => {
                log_message!("Found gen type with type {}.", SynHelper::get_str(inner.clone()).as_str());
                return Some(inner.clone())
            }
            [..] => {
                log_message!("Bean type path did not match any conditions.");
                return None;
            }
        }
    }
}

#[derive(Clone)]
pub struct Bean {
    pub struct_type: Option<Type>,
    pub struct_found: Option<ItemStruct>,
    pub traits_impl: Vec<AutowireType>,
    pub enum_found: Option<ItemEnum>,
    // A reference to another DepImpl - the id is the Type.
    pub deps_map: Vec<DepType>,
    pub id: String,
    pub path_depth: Vec<String>,
    pub profile: Vec<ProfileBuilder>,
    pub ident: Option<Ident>,
    pub fields: Vec<Fields>,
    pub bean_type: Option<BeanType>,
    pub mutable: bool,
    pub aspect_info: Vec<AspectInfo>
}

impl Eq for Bean {}

impl PartialEq<Self> for Bean {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl PartialOrd<Self> for Bean {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Bean {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Default for Bean {
    fn default() -> Self {
        Self {
            struct_type: None,
            struct_found: None,
            traits_impl: vec![],
            enum_found: None,
            deps_map: vec![],
            id: String::default(),
            path_depth: vec![],
            profile: vec![],
            ident: None,
            fields: vec![],
            bean_type: None,
            mutable: false,
            aspect_info: vec![],
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BeanDefinitionType {
    Abstract {
        bean: Bean,
        dep_type: AutowireType
    }, Concrete {
        bean: Bean,
    }
}
