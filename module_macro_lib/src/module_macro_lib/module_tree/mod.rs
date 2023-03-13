use lazy_static::lazy_static;
use std::any::{Any, TypeId};
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, LinkedList};
use std::fmt::{Debug, Formatter, Pointer};
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use syn::{Attribute, Block, Data, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, ImplItem, ImplItemMethod, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Lifetime, parse, parse_macro_input, parse_quote, Path, QSelf, Stmt, TraitItem, Type, TypeArray, TypePath};
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
use module_macro_codegen::aspect::MethodAdviceAspectCodegen;
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

#[derive(Default, Clone)]
pub struct AspectInfo {
    pub(crate) method_advice_aspect: MethodAdviceAspectCodegen,
    pub(crate) method: Option<ImplItemMethod>,
    pub(crate) args: Vec<(Ident, Type)>,
    /// This is the block before any aspects are added.
    pub(crate) original_fn_logic: Option<Block>,
    pub(crate) return_type: Option<Type>,
    pub(crate) method_after: Option<ImplItemMethod>,
    pub(crate) mutable: bool,
    pub(crate) advice_chain: Vec<MethodAdviceChain>
}

#[derive(Default, Clone)]
pub struct MethodAdviceChain {
    pub before_advice: Option<Block>,
    pub after_advice: Option<Block>,
    pub proceed_statement: Option<Stmt>
}

impl MethodAdviceChain {
    pub(crate) fn new(method_advice: &MethodAdviceAspectCodegen) -> Self {
        Self {
            before_advice: method_advice.before_advice.clone(),
            after_advice: method_advice.after_advice.clone(),
            proceed_statement: method_advice.proceed_statement.clone(),
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

#[derive(Clone)]
pub struct BeanPath {
    pub(crate) path_segments: Vec<BeanPathParts>
}

impl BeanPath {

    pub(crate) fn is_mutable(&self) -> bool {
        self.path_segments.iter().any(|p| p.is_mutable())
    }

    pub(crate) fn is_not_mutable(&self) -> bool {
        self.path_segments.iter().all(|p| !p.is_mutable())
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

/**
Will be annotated with #[bean] and #[singleton], #[prototype] as provided factory functions.
 **/
pub struct ModulesFunctions {
    pub fn_found: FunctionType,
    pub path: Vec<String>
}

#[derive(Clone)]
pub struct FunctionType {
    pub(crate) item_fn: ItemFn,
    pub(crate) qualifier: Option<String>,
    pub(crate) fn_type: Option<Type>,
    pub(crate) bean_type: BeanType
}

#[derive(Clone)]
pub struct AutowireType {
    pub item_impl: ItemImpl,
    pub profile: Vec<Profile>,
    pub path_depth: Vec<String>,
    pub qualifiers: Vec<String>
}

#[derive(Clone, Eq, Ord, PartialOrd, PartialEq, Hash, Debug)]
pub struct Profile {
    pub profile: String,
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
    pub bean_type_path: Option<BeanPath>,
    pub is_abstract: Option<bool>
}

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


#[derive(Clone)]
pub struct BeanDefinition {
    pub qualifier: Option<String>,
    pub bean_type_type: Option<Type>,
    pub bean_type_ident: Option<Ident>,
    pub bean_type: BeanType
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
            mutable: false,
            aspect_info: vec![],
        }
    }
}

#[derive(Default, Clone)]
pub struct Trait {
    pub trait_type: Option<ItemTrait>,
    pub trait_path: Vec<String>
}

impl Trait {
    pub fn new(trait_type: ItemTrait, path: Vec<String>) -> Self {
        Self {
            trait_type: Some(trait_type),
            trait_path: path,
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




