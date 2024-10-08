use syn::{Attribute, Fields, Generics, ItemEnum, ItemStruct, ItemUse, parse2, parse_str, Path, Type};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use quote::ToTokens;
use proc_macro2::{Ident, Span};
use syn::__private::str;
use codegen_utils::syn_helper::SynHelper;
use crate::dependency::{DependencyDescriptor, DependencyMetadata};

use crate::functions::ModulesFunctions;
use knockoff_logging::*;
use std::sync::Mutex;
use crate::logger_lazy;
import_logger!("bean.rs");
use enum_fields::EnumFields;
use string_utils::strip_whitespace;
use crate::bean_dependency_path_parser::BeanDependencyPathParser;
use crate::profile_tree::ProfileBuilder;


#[derive(Clone, Default, Debug)]
pub enum AbstractionLevel {
    Abstract,
    #[default]
    Concrete,
}


#[derive(Clone, Debug)]
pub enum BeanType {
    // contains the identifier and the qualifier as string
    Singleton(AbstractionLevel),
    Prototype(AbstractionLevel),
}

impl BeanPathParts {
    pub fn is_mutable(&self) -> bool {
        match self {
            BeanPathParts::PhantomType {..} => {
                false
            }
            BeanPathParts::ArcType { .. } => {
                false
            }
            BeanPathParts::ArcMutexType { inner_ty: arc_mutex_inner_type, outer_path: outer_type , ..} => {
                log_message!("Found mutex for {}, returning true!", SynHelper::get_str(arc_mutex_inner_type.clone()));
                log_message!("Found mutex, returning true!");
                true
            }
            BeanPathParts::MutexType { inner_ty: mutex_type_inner_type, outer_path: outer_type , ..} => {
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
            BeanPathParts::BoxType { .. } => {
                false
            }
        }
    }
}


#[derive(Clone, EnumFields)]
pub enum BeanPathParts {
    ArcType {
        inner_arc_ty: Type,
        outer_arc_pth: Path,
        arc_ident: Option<Ident>
    },
    PhantomType {
        bean_path_parts_phantom_ty: Box<BeanPathParts>,
        phantom_ty_ident: Option<Ident>
    },
    BoxType {
        inner_ty: Type,
        ident: Option<Ident>
    },
    ArcMutexType {
        inner_ty: Type,
        outer_path: Path,
        ident: Option<Ident>
    },
    MutexType {
        inner_ty: Type,
        outer_path: Path,
        ident: Option<Ident>
    },
    //TODO: add recursion for return type to create additional bean path part
    FnType {
        inner_tys: Vec<Type>,
        return_type_opt: Option<Type>,
        ident: Option<Ident>
    },
    QSelfType {
        inner_ty: Type,
        ident: Option<Ident>
    },
    BindingType {
        inner_ty: Type,
        ident: Option<Ident>
    },
    GenType {
        gen_type: Type,
        inner_ty_opt: Option<Type>,
        outer_ty_opt: Option<Type>,
        ident: Option<Ident>
    },
}

#[derive(Clone)]
pub struct BeanPathHead {
    /// The full type path
    pub gen_type_path: Option<syn::Path>,
    /// Provides the identifier for the head.
    pub head_ident: Option<Ident>,
    /// container for if this is dyn, in which case cannot be Ident.
    pub abstract_type: Option<Type>
}


#[derive(Clone)]
pub struct BeanPath {
    pub path_segments: Vec<BeanPathParts>,
    pub head: BeanPathHead
}


pub struct BeanPathParseResult {
    pub tys: HashMap<u32, Type>
}


pub fn get_concrete_type_as_ident(concrete_type: &Option<Type>, ident_type: &Option<Ident>) -> Option<Ident> {
    if concrete_type.as_ref().is_none() && ident_type.as_ref().is_none()  {
        None
    }
    else {
        concrete_type
            .as_ref()
            .map(|t| {
                t
            })
            .map(|t| {
                let parsed = parse2::<Ident>(t.to_token_stream());
                if parsed.is_ok() {
                    Some(Ident::new(t.to_token_stream().to_string().as_str(), Span::call_site()))
                } else {
                    None
                }
            })
            .flatten()
            .or(ident_type.as_ref().map(|i| i.clone()))
    }
}

pub fn get_abstract_type(bean_type: &DependencyDescriptor) -> Option<Type> {
    if bean_type.item_impl.is_some() {
        info!("Testing if {:?} has abstract type for {:?}", SynHelper::get_str(&bean_type.item_impl.as_ref().unwrap()), bean_type);
    }
    let abstract_type = bean_type.item_impl
        .as_ref()
        .map(|item_impl| {
            info!("Getting abstract type for {:?}", SynHelper::get_str(item_impl));
            item_impl.trait_.as_ref()
                .map(|f| BeanDependencyPathParser::parse_path_to_bean_path(&f.1))
        })
        .flatten()
        .or_else(|| {
            bean_type.abstract_type.clone()
        })
        .map(|bean_type| {
            bean_type.get_inner_type()
        })
        .flatten();
    if abstract_type.is_some() {
        info!("Found abstract type: {:?}", SynHelper::get_str(abstract_type.as_ref().unwrap()));
    } else {
        info!("Could not find abstract type for {:?}", bean_type);
    }
    abstract_type
}


impl BeanPath {

    // pub fn get_tys(&self) -> BeanPathParseResult {
    //     let mut out_values = HashMap::new();
    //     info!("Getting tys for {:?}", self);
    //     let mut count = 0;
    //     let _ = self.path_segments.iter()
    //         .for_each(|bp| {
    //             Self::add_segment(&mut out_values, count, bp);
    //         });
    //     // info!("Finished getting tys with {} concrete and {} abstract", concrete.len(), out_values.len());
    //     BeanPathParseResult {
    //         tys: out_values
    //     }
    // }
    //
    // fn add_segment(mut abstract_values: &mut HashMap<u32, Type>, mut count: u32, bp: &BeanPathParts) {
    //     let p = bp.ident().clone();
    //     let inner = bp.inner_ty_opt().cloned()
    //         .or(Some(bp.inner_ty().cloned()))
    //         .flatten();
    //     let inner_ts = inner.as_ref().map(|i| i.to_token_stream().to_string());
    //     let p_ts = p.as_ref().map(|i| i.to_token_stream().to_string());
    //     if inner_ts.as_ref().is_some() && p_ts.as_ref().is_some() {
    //         if inner_ts.as_ref().unwrap() == p_ts.as_ref().unwrap() {
    //             let value = p.clone().unwrap().into_token_stream();
    //             info!("Parsing {:?} to path.", SynHelper::get_str(&value));
    //             abstract_values.insert(count, parse2(value).unwrap());
    //             count += 1;
    //         }
    //     } else if inner.as_ref().is_some() {
    //         abstract_values.insert(count, inner.unwrap());
    //         count += 1;
    //         info!("{:?} is next ty", &inner_ts);
    //     }
    // }

    pub fn patterns_to_match<'a>(in_type: &Type, path: &syn::Path) -> (String, String, Vec<BeanPathParts>, Vec<String>) {
        let (match_ts, path_str_to_match) = Self::get_to_match(in_type, path);
        info!("Parsing {} path.", path_str_to_match);
        let mut bean_parts = vec![];
        let pattern = Self::get_patterns(&path_str_to_match);
        (match_ts, path_str_to_match, bean_parts, pattern)
    }

    pub fn get_to_match(in_type: &Type, path: &syn::Path) -> (String, String) {
        let match_ts = in_type.to_token_stream().to_string();
        let path_str = path.to_token_stream().to_string();
        let path_str_to_match = path_str;
        (match_ts, path_str_to_match)
    }

    pub fn get_patterns(path_str: &String) -> Vec<String> {
        assert!(!path_str.contains("Fn") && !path_str.contains("FnMut") && !path_str.contains("FnOnce"),
                "Functions are not supported for injection currently.");
        let pattern = path_str.split("<").into_iter()
            .flat_map(|s| s.split(">").into_iter())
            .flat_map(|s| strip_whitespace(s).into_iter())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        pattern
    }

}

impl BeanPathParts {


    fn get_matcher(&self) -> Vec<String> {
        match self {
            BeanPathParts::ArcType { inner_arc_ty: arc_inner_types, outer_arc_pth: outer_type , ..} => {
                vec![arc_inner_types.to_token_stream().to_string(), outer_type.to_token_stream().to_string()]
            }
            BeanPathParts::PhantomType { bean_path_parts_phantom_ty: inner , ..} => {
                vec![]
            }
            BeanPathParts::ArcMutexType { inner_ty: arc_mutex_inner_type, outer_path: outer_type , ..} => {
                vec![arc_mutex_inner_type.to_token_stream().to_string(), outer_type.to_token_stream().to_string()]
            }
            BeanPathParts::MutexType { inner_ty: mutex_type_inner_type, outer_path: outer_type , ..} => {
                vec![mutex_type_inner_type.to_token_stream().to_string(), outer_type.to_token_stream().to_string()]
            }
            BeanPathParts::FnType { inner_tys: input_types, return_type_opt: return_type , ..} => {
                vec![input_types.iter().map(|i| i.to_token_stream().to_string()).collect::<Vec<String>>().join(", "),
                     return_type.to_token_stream().to_string()]
            }
            BeanPathParts::QSelfType { inner_ty: q_self, .. } => {
                vec![q_self.to_token_stream().to_string()]
            }
            BeanPathParts::BindingType { inner_ty: associated_type, .. } => {
                vec![associated_type.to_token_stream().to_string()]
            }
            BeanPathParts::GenType { gen_type: inner , inner_ty_opt: inner_ty, ..} => {
                vec![inner.to_token_stream().to_string(), SynHelper::get_str(inner_ty)]
            }
            BeanPathParts::BoxType { inner_ty: inner , .. } => {
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
        match &self.path_segments.clone()[..] {
            [ BeanPathParts::ArcMutexType{ inner_ty: arc_mutex_inner_type, outer_path: out, .. },
              BeanPathParts::MutexType { inner_ty: mutex_type_inner_type, outer_path: outer_type , .. },
              BeanPathParts::GenType { gen_type: inner , inner_ty_opt: inner_ty, ..} ] => {
                log_message!("Found arc mutex type with type {} and gen type {}.", SynHelper::get_str(arc_mutex_inner_type.clone()).as_str(), SynHelper::get_str(mutex_type_inner_type.clone()));
                return Some(inner.clone());
            }
            [ BeanPathParts::PhantomType { bean_path_parts_phantom_ty: inner , .. }, .. ] => {
                if let BeanPathParts::PhantomType { bean_path_parts_phantom_ty: inner , .. }  = inner.as_ref() {
                    if let BeanPathParts::GenType { inner_ty_opt: inner, gen_type, .. } = inner.deref().deref() {
                        return inner.clone()
                    }
                }
                None
            }
            [ BeanPathParts::MutexType{ inner_ty: mutex_type_inner_type, outer_path: outer_type , .. },
            BeanPathParts::GenType { gen_type: inner, .. } ] => {
                log_message!("Found mutex type with type {} and gen type {}.", SynHelper::get_str(mutex_type_inner_type.clone()), SynHelper::get_str(inner.clone()));
                return Some(inner.clone());
            }
            [ BeanPathParts::ArcType{ inner_arc_ty: arc_inner_types, outer_arc_pth: outer_type , .. },
            BeanPathParts::GenType { gen_type: inner , ..}, ..] => {
                log_message!("Found arc type with type {} and gen type {}.", SynHelper::get_str(arc_inner_types.clone()).as_str(), SynHelper::get_str(inner.clone()));
                return Some(inner.clone());
            }
            [ BeanPathParts::ArcMutexType{ inner_ty: arc_mutex_inner_type, outer_path: out, .. },
            BeanPathParts::MutexType { inner_ty: mutex_type_inner_type, outer_path: outer_type , .. }] => {
                log_message!("Found arc mutex type with type {} and gen type {}.", SynHelper::get_str(arc_mutex_inner_type.clone()).as_str(), SynHelper::get_str(mutex_type_inner_type.clone()));
                return Some(mutex_type_inner_type.clone());
            }
            [ BeanPathParts::MutexType { inner_ty: mutex_type_inner_type, outer_path: outer_type , .. }, ..  ] => {
                log_message!("Found fn and mutex type with type {}.", SynHelper::get_str(mutex_type_inner_type).as_str());
                return Some(mutex_type_inner_type.clone());
            }
            [ BeanPathParts::FnType { inner_tys: input_types, return_type_opt: return_type , .. }, ..  ] => {
                log_message!("Found fn and mutex type with type {}.", SynHelper::get_str(return_type.clone()).as_str());
                return return_type.clone();
            }
            [ BeanPathParts::QSelfType { inner_ty: q_self, .. }, .. ] => {
                log_message!("Found qself type with type {}.", SynHelper::get_str(q_self.clone()).as_str());
                return Some(q_self.clone());
            }
            [ BeanPathParts::BindingType { inner_ty: associated_type, .. }, .. ] => {
                log_message!("Found binding type with type {}.", SynHelper::get_str(associated_type.clone()).as_str());
                return None;
            }
            [ BeanPathParts::ArcType{ inner_arc_ty: arc_inner_types, outer_arc_pth: outer_type , .. }, .. ] => {
                log_message!("Found arc type with type {}.", SynHelper::get_str(arc_inner_types.clone()).as_str());
                return Some(arc_inner_types.clone());
            }
            [ BeanPathParts::GenType { gen_type: inner , ..} ] => {
                log_message!("Found gen type with type {}.", SynHelper::get_str(inner.clone()).as_str());
                return Some(inner.clone())
            }
            [ BeanPathParts::GenType { gen_type: inner , ..}, ..  ] => {
                log_message!("Found gen type with type {}.", SynHelper::get_str(inner.clone()).as_str());
                return Some(inner.clone())
            }
            [BeanPathParts::BoxType {..}, BeanPathParts::GenType {gen_type, ..}, ..] => {
                return Some(gen_type.clone())
            }
            [ BeanPathParts::ArcMutexType {..}, BeanPathParts::MutexType {..},
            BeanPathParts::BoxType { inner_ty: inner , .. }, ..] => {
                return Some(inner.clone())
            }
            [ BeanPathParts::ArcMutexType { .. }, BeanPathParts::MutexType { .. },
            BeanPathParts::BoxType { .. }, BeanPathParts::GenType { gen_type , ..}, .. ] => {
                return Some(gen_type.clone())
            }
            [ BeanPathParts::GenType {gen_type, ..}, ..] => {
                return Some(gen_type.clone());
            }
            e => {
                if e.len() == 0 {
                    None
                } else if let BeanPathParts::PhantomType { bean_path_parts_phantom_ty: inner , .. } = &e[1] {
                    if let BeanPathParts::GenType { gen_type, .. } = inner.deref().deref() {
                        Some(gen_type.clone())
                    } else {
                        None
                    }
                } else {
                    panic!("Bean type path {:?} did not match any conditions.", &self);
                }
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
    pub factory_fn: Option<ModulesFunctions>,
    pub declaration_generics: Option<Generics>,
    pub qualifiers: Vec<String>
}

impl BeanDefinition {
    pub fn is_constructable(&self) -> bool {
        !self.has_fn("new")
    }

    pub fn has_default(&self) -> bool {
        self.iter_attrs()
            .map(|a| SynHelper::get_attr_from_vec(a, &vec!["Default"]))
            .is_some() || self.has_fn("default")
    }

    pub fn has_attribute(&self, matcher: &dyn Fn(&Attribute) -> bool) -> bool {
        self.iter_attrs().iter().flat_map(|a| a.iter())
            .any(|a| matcher(a))
    }

    pub fn iter_attrs(&self) -> Option<&Vec<Attribute>> {
        if let Some(s) = &self.struct_found {
            Some(&s.attrs)
        }  else if let Some(s) = &self.enum_found {
            Some(&s.attrs)
        } else if let Some(s) = &self.factory_fn {
            Some(&s.fn_found.item_fn.attrs)
        } else {
            None
        }
    }

    fn has_fn(&self, fn_to_have: &str) -> bool {
        info!("Checking if {:?} is {}", self, &fn_to_have);
        if self.traits_impl.iter().any(|b| b.has_fn_named(fn_to_have)) {
            info!("found that {:?} had {}", self, &fn_to_have);
            true
        } else {
            info!("found that {:?} did not have {}", self, &fn_to_have);
            false
        }
    }
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
            factory_fn: None,
            deps_map: vec![],
            declaration_generics: None,
            qualifiers: vec![],
        }
    }
}

#[derive(Clone, Eq, PartialEq, EnumFields)]
pub enum BeanDefinitionType {
    Abstract {
        /// Abstract is the trait impls - the abstract Bean Definition could be out of date. This is
        /// read-only and it only contains changes up to when it was copied over.
        bean: BeanDefinition,
        dep_type: DependencyDescriptor,
    }, Concrete {
        bean: BeanDefinition,
    }
}
