use syn::{Attribute, Field, Fields, ItemEnum, ItemImpl, ItemStruct, ItemUse, Lifetime, parse_str, Path, Stmt, Type, TypeArray};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::fmt;
use codegen_utils::syn_helper;
use quote::ToTokens;
use proc_macro2::{Ident, TokenStream};
use syn::__private::str;
use syn::token::Use;
use codegen_utils::syn_helper::SynHelper;
use crate::aspect::AspectInfo;
use crate::dependency::{ArgDepType, AutowiredField, DependencyDescriptor, DependencyMetadata, FieldDepType};

use knockoff_logging::{initialize_log, use_logging};
use crate::functions::ModulesFunctions;
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

#[derive(Clone, PartialEq)]
pub struct BeanPath {
    pub path_segments: Vec<BeanPathParts>
}

impl Eq for BeanPath {}

impl BeanPathParts {
    fn get_matcher(&self) -> Vec<String> {
        match self {
            BeanPathParts::ArcType { arc_inner_types, outer_type } => {
                vec![arc_inner_types.to_token_stream().to_string(), outer_type.to_token_stream().to_string()]
            }
            BeanPathParts::ArcMutexType { arc_mutex_inner_type, outer_type } => {
                vec![arc_mutex_inner_type.to_token_stream().to_string(), outer_type.to_token_stream().to_string()]
            }
            BeanPathParts::MutexType { mutex_type_inner_type, outer_type } => {
                vec![mutex_type_inner_type.to_token_stream().to_string(), outer_type.to_token_stream().to_string()]
            }
            BeanPathParts::FnType { input_types, return_type } => {
                vec![input_types.iter().map(|i| i.to_token_stream().to_string()).collect::<Vec<String>>().join(", "),
                     return_type.to_token_stream().to_string()]
            }
            BeanPathParts::QSelfType { q_self } => {
                vec![q_self.to_token_stream().to_string()]
            }
            BeanPathParts::BindingType { associated_type } => {
                vec![associated_type.to_token_stream().to_string()]
            }
            BeanPathParts::GenType { inner } => {
                vec![inner.to_token_stream().to_string()]
            }
        }
    }
}

impl PartialEq<Self> for BeanPathParts {
    fn eq(&self, other: &Self) -> bool {
        self.get_matcher() == other.get_matcher()
    }
}

impl Eq for BeanPathParts {

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
pub struct BeanDefinition {
    pub struct_type: Option<Type>,
    pub struct_found: Option<ItemStruct>,
    pub traits_impl: Vec<DependencyDescriptor>,
    pub enum_found: Option<ItemEnum>,
    pub deps_map: Vec<DependencyMetadata>,
    pub id: String,
    pub path_depth: Vec<String>,
    pub profile: Vec<ProfileBuilder>,
    pub ident: Option<Ident>,
    pub fields: Vec<Fields>,
    pub bean_type: Option<BeanType>,
    pub mutable: bool,
    pub aspect_info: Vec<AspectInfo>,
    pub factory_fn: Option<ModulesFunctions>,
}

impl BeanDefinition {

    pub fn get_use_stmts(&self) -> HashMap<String, ItemUse> {
        log_message!("Adding use statements for {} with {} dependencies and {} traits.", &self.id, self.deps_map.len(), self.traits_impl.len());
        let mut use_stmts = HashMap::new();
        self.add_self_value(&mut use_stmts);
        self.add_abstract(&mut use_stmts);
        Self::create_use_stmts(&mut use_stmts)
    }

    fn add_self_value(&self, mut use_stmts: &mut HashMap<String, Vec<String>>) {
        self.factory_fn.as_ref()
            .map(|_| self.add_fn_stmt(&mut use_stmts))
            .or_else(|| {
                self.add_fn_stmt(&mut use_stmts);
                None
            });
    }

    fn add_fn_stmt(&self, mut use_stmts: &mut HashMap<String, Vec<String>>)  {
        self.factory_fn.as_ref().map(|f| {
            self.ident.as_ref()
                .map(|fn_ident| use_stmts.insert(fn_ident.to_string().clone(), f.path.clone()))
        });
    }

    fn add_abstract(&self, mut use_stmts: &mut HashMap<String, Vec<String>>) {
        self.traits_impl.iter()
            .for_each(|t| {
                t.abstract_type.as_ref()
                    .map(|a| a.get_inner_type())
                    .map(|a|
                        a.map(|a| use_stmts.insert(a.to_token_stream().to_string(), t.path_depth.clone()))
                    );
            });
    }

    fn add_self_struct(&self, mut use_stmts: &mut HashMap<String, Vec<String>>) {
        self.struct_type.as_ref()
            .map(|s| {
                let ty = s.to_token_stream().to_string();
                log_message!("Adding use statement for self ty: {}.", &ty);
                ty
            })
            .or_else(|| {
                self.ident.as_ref()
                    .map(|i| i.to_string())
            })
            .map(|key| {
                use_stmts.insert(key, self.path_depth.clone());
            });
    }

    fn create_use_stmts(use_stmts: &mut HashMap<String, Vec<String>>) -> HashMap<String, ItemUse> {
        use_stmts.iter().flat_map(|u| {
            if u.1.len() != 0 {
                let mut joined = u.1.join("::");
                joined += "::";
                joined += u.0.as_str();
                let mut use_stmt = "use ".to_string();
                use_stmt += joined.as_str();
                use_stmt += ";";
                return parse_str::<ItemUse>(use_stmt.as_str())
                    .ok()
                    .map(|use_stmt| {
                        vec![(u.0.clone(), use_stmt)]
                    })
                    .or(Some(vec![]))
                    .unwrap();
            } else {
                log_message!("Could not create use statement for {} because path was 0.", u.0);
            }
            vec![]
        }).collect::<HashMap<String, ItemUse>>()
    }
}

impl  Eq for BeanDefinition {}

impl  PartialEq<Self> for BeanDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl  PartialOrd<Self> for BeanDefinition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl  Ord for BeanDefinition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl  Default for BeanDefinition {
    fn default() -> Self {
        Self {
            struct_type: None,
            struct_found: None,
            traits_impl: vec![],
            enum_found: None,
            id: String::default(),
            path_depth: vec![],
            profile: vec![],
            ident: None,
            fields: vec![],
            bean_type: None,
            mutable: false,
            aspect_info: vec![],
            factory_fn: None,
            deps_map: vec![],
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum BeanDefinitionType {
    Abstract {
        bean: BeanDefinition,
        dep_type: DependencyDescriptor
    }, Concrete {
        bean: BeanDefinition,
    }
}
