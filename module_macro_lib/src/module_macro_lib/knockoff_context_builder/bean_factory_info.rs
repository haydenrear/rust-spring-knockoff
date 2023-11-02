use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::Arc;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{Field, Fields, GenericParam, Generics, ImplItem, parse2, Path, PredicateType, Type, TypeParam, TypeParamBound, WherePredicate};
use syn::token::Struct;
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::{BeanDefinition, BeanType};

use module_macro_shared::dependency::{DependencyDescriptor, DependencyMetadata, DepType};
use module_macro_shared::functions::ModulesFunctions;
use module_macro_shared::profile_tree::ProfileBuilder;
use crate::module_macro_lib::bean_parser::bean_dependency_path_parser::BeanDependencyPathParser;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use syn::ext::IdentExt;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("bean_factory_info.rs");

#[derive(Clone, Default)]
pub struct BeansFieldTypeInfo {
    pub(crate) concrete_field_type_info: Vec<AutowirableFieldTypeInfo>,
    pub(crate) concrete_mutable_field_type_info: Vec<MutableFieldInfo>,
    pub(crate) abstract_field_type_info: Vec<AbstractFieldInfo>,
    pub(crate) abstract_mutable_field_type_info: Vec<MutableAbstractFieldInfo>,
}

#[derive(Clone, Default)]
pub struct BeanFactoryInfo {
    pub(crate) singleton_field_type_info: BeansFieldTypeInfo,
    pub(crate) prototype_field_type_info: BeansFieldTypeInfo,
    pub(crate) default_field_info: Vec<DefaultFieldInfo>,
    pub(crate) concrete_type: Option<Type>,
    pub(crate) is_enum: bool,
    pub(crate) is_default: bool,
    pub(crate) abstract_type: Option<Type>,
    pub(crate) ident_type: Option<Ident>,
    pub(crate) profile: Option<ProfileBuilder>,
    pub(crate) factory_fn: Option<ModulesFunctions>,
    pub(crate) constructable: bool
}

#[derive(Clone)]
pub struct AutowirableFieldTypeInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident
}

#[derive(Clone)]
pub struct DefaultFieldInfo {
    pub(crate) field_type: Type,
    pub(crate) field_ident: Ident
}

impl Debug for DefaultFieldInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("DefaultFieldInfo: ")?;
        f.write_str(format!("Field ident: {}, ", SynHelper::get_str(&self.field_ident).as_str()).as_str())?;
        f.write_str(format!("Field type: {}, ", SynHelper::get_str(&self.field_type).as_str()).as_str())?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct DefaultFieldTypeInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident
}

#[derive(Clone)]
pub struct MutableFieldInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident
}

#[derive(Clone)]
pub struct AbstractFieldInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident,
    qualifier: Option<String>,
    profile: Option<String>,

}

#[derive(Clone)]
pub struct MutableAbstractFieldInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident,
    qualifier: Option<String>,
    profile: Option<String>
}

type ConcreteFieldIdents = Vec<Ident>;
type ConcreteAutowirableFieldType = Vec<Type>;
type ConcreteFieldTypes = Vec<Type>;

type ConcreteMutableFieldIdents = Vec<Ident>;
type ConcreteMutableAutowirableFieldType = Vec<Type>;
type ConcreteMutableFieldTypes = Vec<Type>;


type AbstractFieldIdents = Vec<Ident>;
type AbstractAutowirableFieldType = Vec<Type>;
type AbstractFieldTypes = Vec<Type>;

type AbstractMutableFieldIdents = Vec<Ident>;
type AbstractMutableAutowirableFieldType = Vec<Type>;
type AbstractMutableFieldTypes = Vec<Type>;


type FieldTypes = (ConcreteFieldIdents, ConcreteAutowirableFieldType, ConcreteFieldTypes,
                   ConcreteMutableFieldIdents,ConcreteMutableAutowirableFieldType, ConcreteMutableFieldTypes,
                   AbstractFieldIdents, AbstractAutowirableFieldType,AbstractFieldTypes,
                   AbstractMutableFieldIdents, AbstractMutableAutowirableFieldType, AbstractMutableFieldTypes);

impl BeanFactoryInfo {

    pub(crate) fn get_abstract_type(bean_type: &DependencyDescriptor) -> Option<Type> {
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

    pub(crate) fn get_profile_ident(&self) -> Ident {
        self.profile.as_ref()
            .map(|p| Ident::new(p.profile.as_str(), Span::call_site()))
            .or(Some(Ident::new(ProfileBuilder::default().profile.as_str(), Span::call_site())))
            .unwrap()
    }

    pub(crate) fn get_concrete_type(&self) -> Option<Type> {
        assert!(self.concrete_type.is_some() || self.ident_type.is_some(),
                "Could not retrieve concrete type when creating concrete bean factory.");
        self.concrete_type.clone()
    }

    pub(crate) fn get_concrete_type_as_ident(&self) -> Option<Ident> {
        assert!(self.concrete_type.is_some() || self.ident_type.is_some(),
                "Could not retrieve concrete type when creating concrete bean factory.");
        self.ident_type.as_ref().map(|i| i.clone())
    }

    pub(crate) fn get_field_prototype_types(&self)-> FieldTypes {
        let prototype_field_type_info = &self.prototype_field_type_info;
        let field_types = prototype_field_type_info.concrete_field_type_info.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let field_idents = prototype_field_type_info.concrete_field_type_info.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let field_concrete = prototype_field_type_info.concrete_field_type_info.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();

        let prototype_mutable_field_type_info = &prototype_field_type_info.concrete_mutable_field_type_info;
        let prototype_abstract_field_type_info = &prototype_field_type_info.abstract_field_type_info;
        let prototype_mutable_abstract_field_type_info = &prototype_field_type_info.abstract_mutable_field_type_info;

        let mutable_field_idents = prototype_mutable_field_type_info.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let mutable_field_types = prototype_mutable_field_type_info.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_field_concrete = prototype_mutable_field_type_info.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let abstract_field_ident = prototype_abstract_field_type_info.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let abstract_field_types = prototype_abstract_field_type_info.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let abstract_field_concrete = prototype_abstract_field_type_info.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_abstract_field_ident = prototype_mutable_abstract_field_type_info.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let mutable_abstract_field_types = prototype_mutable_abstract_field_type_info.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_abstract_field_concrete = prototype_mutable_abstract_field_type_info.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();

        (field_idents, field_types,  field_concrete,
         mutable_field_idents, mutable_field_types, mutable_field_concrete,
         abstract_field_ident, abstract_field_types, abstract_field_concrete,
         mutable_abstract_field_ident, mutable_abstract_field_types, mutable_abstract_field_concrete)
    }

    pub(crate) fn get_field_singleton_types(&self)-> FieldTypes {
        let singleton_field_type_info = &self.singleton_field_type_info;
        let field_types = singleton_field_type_info.concrete_field_type_info.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let field_idents = singleton_field_type_info.concrete_field_type_info.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let field_concrete = singleton_field_type_info.concrete_field_type_info.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();

        let singleton_mutable_field_type_info = &singleton_field_type_info.concrete_mutable_field_type_info;
        let singleton_abstract_field_type_info = &singleton_field_type_info.abstract_field_type_info;
        let singleton_mutable_abstract_field_type_info = &singleton_field_type_info.abstract_mutable_field_type_info;

        let mutable_field_idents = singleton_mutable_field_type_info.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let mutable_field_types = singleton_mutable_field_type_info.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_field_concrete = singleton_mutable_field_type_info.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let abstract_field_ident = singleton_abstract_field_type_info.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let abstract_field_types = singleton_abstract_field_type_info.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let abstract_field_concrete = singleton_abstract_field_type_info.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_abstract_field_ident = singleton_mutable_abstract_field_type_info.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let mutable_abstract_field_types = singleton_mutable_abstract_field_type_info.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_abstract_field_concrete = singleton_mutable_abstract_field_type_info.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();

        (field_idents, field_types,  field_concrete,
         mutable_field_idents, mutable_field_types, mutable_field_concrete,
         abstract_field_ident, abstract_field_types, abstract_field_concrete,
         mutable_abstract_field_ident, mutable_abstract_field_types, mutable_abstract_field_concrete)
    }
}

pub trait BeanFactoryInfoFactory<T> {

    fn create_bean_factory_info(bean: &T) -> Vec<BeanFactoryInfo>;

    fn get_mutable_singleton_field_ids(token_type: &BeanDefinition) -> Vec<MutableFieldInfo> {
        Self::get_field_ids::<MutableFieldInfo>(token_type, &Self::create_mutable_singleton_dep_type)
    }

    fn get_mutable_prototype_field_ids(token_type: &BeanDefinition) -> Vec<MutableFieldInfo> {
        Self::get_field_ids::<MutableFieldInfo>(token_type, &Self::create_mutable_prototype_dep_type)
    }

    fn get_prototype_field_ids(bean: &BeanDefinition) -> Vec<AutowirableFieldTypeInfo> {
        Self::get_field_ids::<AutowirableFieldTypeInfo>(bean, &Self::create_prototype_dep_type)
    }

    fn get_singleton_field_ids(bean: &BeanDefinition) -> Vec<AutowirableFieldTypeInfo> {
        Self::get_field_ids::<AutowirableFieldTypeInfo>(bean, &Self::create_singleton_dep_type)
    }

    fn get_abstract_singleton_field_ids(bean: &BeanDefinition) -> Vec<AbstractFieldInfo> {
        Self::get_field_ids::<AbstractFieldInfo>(bean, &Self::create_abstract_singleton_dep_type)
    }

    fn get_abstract_prototype_field_ids(bean: &BeanDefinition) -> Vec<AbstractFieldInfo> {
        Self::get_field_ids::<AbstractFieldInfo>(bean, &Self::create_abstract_prototype_dep_type)
    }

    fn get_abstract_mutable_prototype_field_ids(bean: &BeanDefinition) -> Vec<MutableAbstractFieldInfo> {
        Self::get_field_ids::<MutableAbstractFieldInfo>(
            bean,
            &Self::create_mutable_prototype_abstract_dep_type
        )
    }

    fn get_abstract_mutable_singleton_field_ids(bean: &BeanDefinition) -> Vec<MutableAbstractFieldInfo> {
        Self::get_field_ids::<MutableAbstractFieldInfo>(
            bean,
            &Self::create_mutable_singleton_abstract_dep_type
        )
    }

    fn get_default_fields(bean: &BeanDefinition) -> Vec<DefaultFieldInfo> {
        if bean.fields.len() > 1 && !bean.is_constructable() && !bean.has_default(){
            error!(
                "Type had more than one set of fields and no new() function provided. Enum with \
                multiple types of fields is not ready to be autowired!"
            );
            return vec![];
        } else if bean.fields.len() == 0 {
            return vec![];
        }

        info!("Setting default fields {} and {} for {:?}", bean.deps_map.len(), bean.fields[0].len(), bean);
        match &bean.fields[0] {
            Fields::Named(n) => {
                info!("Has {} fields.", n.named.len());
                n.named.iter()
                    .filter(|f|
                        SynHelper::get_attr_from_vec(
                            &f.attrs,
                            &vec!["autowired"]
                        ).is_none()
                    )
                    .flat_map(|f| {
                        if f.ident.as_ref().is_some() {
                            vec![DefaultFieldInfo{
                                field_type: f.ty.clone(),
                                field_ident: f.ident.as_ref().unwrap().clone()
                            }]
                        } else {
                            let f = SynHelper::get_str(f);
                            info!("Failed to parse {:?}, as its field ident was nonexistent.", f);
                            vec![]
                        }
                    } )
                    .map(|def| {
                        info!("Found default fields {:?}", def);
                        def
                    })
                    .collect::<Vec<DefaultFieldInfo>>()
            }
            _ => {
                vec![]
            }
        }
    }

    fn create_mutable_singleton_dep_type(dep_type: &DependencyMetadata) -> Option<MutableFieldInfo> {
        if dep_type.is_abstract().or(Some(false)).unwrap() {
            return None;
        }
        info!("testing if {:?} is singleton.", dep_type.bean_type());
        if matches!(dep_type.bean_type(), Some(BeanType::Singleton(_))) {
            info!("{:?} was singleton.", dep_type.bean_type());
            Self::create_mutable_dep_type(dep_type)
        } else {
            None
        }
    }

    fn create_mutable_prototype_dep_type(dep_type: &DependencyMetadata) -> Option<MutableFieldInfo> {
        if dep_type.is_abstract().or(Some(false)).unwrap() {
            return None;
        }
        info!("testing if {:?} is prototype.", dep_type.bean_type());
        if matches!(dep_type.bean_type(), Some(BeanType::Prototype(_))) {
            info!("{:?} was prototype.", dep_type.bean_type());
            Self::create_mutable_dep_type(dep_type)
        } else {
            None
        }
    }

    fn create_mutable_dep_type(dep_type: &DependencyMetadata) -> Option<MutableFieldInfo> {
        dep_type.bean_type_path()
            .as_ref()
            .filter(|d| d.is_mutable())
            .map(|type_path| type_path.get_autowirable_type())
            .flatten()
            .map(|field_type| {
                let concrete_field_type = dep_type.dep_type_concrete_type()
                    .as_ref()
                    .cloned()
                    .or(Some(field_type.clone())).unwrap();
                let field_ident = dep_type.dep_type_field_ident();
                MutableFieldInfo {
                    concrete_field_type,
                    field_type,
                    field_ident,
                }
            })
    }

    fn create_singleton_dep_type(dep_type: &DependencyMetadata) -> Option<AutowirableFieldTypeInfo> {
        if !dep_type.is_abstract().or(Some(false)).unwrap() && matches!(dep_type.bean_type(), Some(BeanType::Singleton(_))) {
            Self::create_dep_type(dep_type)
        } else {
            None
        }
    }

    fn create_prototype_dep_type(dep_type: &DependencyMetadata) -> Option<AutowirableFieldTypeInfo> {
        if !dep_type.is_abstract().or(Some(false)).unwrap() && matches!(dep_type.bean_type(), Some(BeanType::Prototype(_))) {
            Self::create_dep_type(dep_type)
        } else {
            None
        }
    }

    fn create_dep_type(dep_type: &DependencyMetadata) -> Option<AutowirableFieldTypeInfo> {
        dep_type.bean_type_path()
            .as_ref()
            .filter(|d| d.is_not_mutable())
            .map(|type_path| type_path.get_autowirable_type())
            .flatten()
            .map(|field_type| AutowirableFieldTypeInfo {
                concrete_field_type: dep_type.dep_type_concrete_type().clone().or(Some(field_type.clone())).unwrap(),
                field_type,
                field_ident: dep_type.dep_type_field_ident(),
            })
    }

    fn create_abstract_prototype_dep_type(dep_type: &DependencyMetadata) -> Option<AbstractFieldInfo> {
        if dep_type.is_abstract().or(Some(false)).unwrap() && matches!(dep_type.bean_type(), Some(BeanType::Prototype(_))) {
            if let Some(value) = Self::get_abstract_dep_type(dep_type) {
                return value;
            }
        }
        None
    }

    fn create_abstract_singleton_dep_type(dep_type: &DependencyMetadata) -> Option<AbstractFieldInfo> {
        if dep_type.is_abstract().or(Some(false)).unwrap() && matches!(dep_type.bean_type(), Some(BeanType::Singleton(_))) {
            if let Some(value) = Self::get_abstract_dep_type(dep_type) {
                return value;
            }
        }
        None
    }

    fn get_abstract_dep_type(dep_type: &DependencyMetadata) -> Option<Option<AbstractFieldInfo>> {
        return Some(dep_type.bean_type_path()
            .as_ref()
            .filter(|d| d.is_not_mutable())
            .map(|type_path| type_path.get_autowirable_type())
            .flatten()
            .map(|field_type| AbstractFieldInfo {
                concrete_field_type: dep_type.dep_type_concrete_type().clone().or(Some(field_type.clone())).unwrap(),
                field_type,
                qualifier: dep_type.dep_type_maybe_qualifier().clone(),
                profile: None,
                field_ident: dep_type.dep_type_field_ident(),
            }));
        None
    }

    fn create_mutable_prototype_abstract_dep_type(dep_type: &DependencyMetadata) -> Option<MutableAbstractFieldInfo> {
        if dep_type.is_abstract().or(Some(false)).unwrap() && matches!(dep_type.bean_type(), Some(BeanType::Prototype(_))) {
            if let Some(value) = Self::create_mutable_abstract_dep_type(dep_type) {
                return value;
            }
        }
        None
    }

    fn create_mutable_singleton_abstract_dep_type(dep_type: &DependencyMetadata) -> Option<MutableAbstractFieldInfo> {
        if dep_type.is_abstract().or(Some(false)).unwrap() && matches!(dep_type.bean_type(), Some(BeanType::Singleton(_))) {
            if let Some(value) = Self::create_mutable_abstract_dep_type(dep_type) {
                return value;
            }
        }
        None
    }

    fn create_mutable_abstract_dep_type(dep_type: &DependencyMetadata) -> Option<Option<MutableAbstractFieldInfo>> {
        return Some(dep_type.bean_type_path()
            .as_ref()
            .filter(|d| d.is_mutable())
            .map(|type_path| type_path.get_autowirable_type())
            .flatten()
            .map(|field_type| {
                let concrete_field_type = dep_type.dep_type_concrete_type().clone().or(Some(field_type.clone())).unwrap();
                MutableAbstractFieldInfo {
                    concrete_field_type,
                    field_type,
                    qualifier: dep_type.dep_type_maybe_qualifier().clone(),
                    profile: None,
                    field_ident: dep_type.dep_type_field_ident(),
                }
            }));
        None
    }

    fn get_field_ids<U>(
        token_type: &BeanDefinition,
        creator: &dyn Fn(&DependencyMetadata) -> Option<U>
    ) -> Vec<U> {
        let field_types = token_type.deps_map
            .iter()
            .flat_map(|d| creator(d)
                .map(|item| vec![item])
                .or(Some(vec![]))
                .unwrap()
            )
            .collect::<Vec<U>>();

        field_types
    }

}

/// ConcreteBeanFactoryInfo is the default implementation of the beans... So the Profile is only used
/// when there is an abstract bean that will be different for different Profiles or qualifiers.
pub struct ConcreteBeanFactoryInfo;

impl BeanFactoryInfoFactory<BeanDefinition> for ConcreteBeanFactoryInfo {

    fn create_bean_factory_info(bean: &BeanDefinition) -> Vec<BeanFactoryInfo> {

        log_message!("Creating bean factory info for bean with id {} with has {} dependencies.", &bean.id, bean.deps_map.len());

        // TODO: Fix this - not adding dependencies in some cases.
        let mutable_singleton_field_ids = Self::get_mutable_singleton_field_ids(bean);
        let singleton_field_ids = Self::get_singleton_field_ids(bean);

        let mutable_prototype_field_ids = Self::get_mutable_prototype_field_ids(bean);
        let prototype_field_ids = Self::get_prototype_field_ids(bean);

        let mutable_abstract_singleton_fields = Self::get_abstract_mutable_singleton_field_ids(bean);
        let mutable_abstract_prototype_fields = Self::get_abstract_mutable_prototype_field_ids(&bean);

        let abstract_singleton_fields = Self::get_abstract_singleton_field_ids(bean);
        let abstract_prototype_fields = Self::get_abstract_prototype_field_ids(bean);

        let default_field_info = Self::get_default_fields(bean);

        bean.profile.iter()
            .map(|p| BeanFactoryInfo {
                singleton_field_type_info: BeansFieldTypeInfo {
                    concrete_field_type_info: singleton_field_ids.clone(),
                    concrete_mutable_field_type_info: mutable_singleton_field_ids.clone(),
                    abstract_field_type_info: abstract_singleton_fields.clone(),
                    abstract_mutable_field_type_info: mutable_abstract_singleton_fields.clone(),
                },
                prototype_field_type_info: BeansFieldTypeInfo {
                    concrete_field_type_info: prototype_field_ids.clone(),
                    concrete_mutable_field_type_info: mutable_prototype_field_ids.clone(),
                    abstract_field_type_info: abstract_prototype_fields.clone(),
                    abstract_mutable_field_type_info: mutable_abstract_prototype_fields.clone(),
                },
                default_field_info: default_field_info.clone(),
                concrete_type: bean.struct_type.clone(),
                is_enum: bean.enum_found.is_some(),
                abstract_type: None,
                ident_type: bean.ident.clone(),
                profile: Some(ProfileBuilder::default()),
                factory_fn: bean.factory_fn.clone(),
                constructable: bean.is_constructable(),
                is_default: bean.has_default()
            })
            .collect::<Vec<BeanFactoryInfo>>()
    }
}

pub struct AbstractBeanFactoryInfo;

impl BeanFactoryInfoFactory<(BeanDefinition, DependencyDescriptor, ProfileBuilder)> for AbstractBeanFactoryInfo {

    fn create_bean_factory_info(bean_type: &(BeanDefinition, DependencyDescriptor, ProfileBuilder)) -> Vec<BeanFactoryInfo> {
        let bean = &bean_type.0;

        let abstract_type = BeanFactoryInfo::get_abstract_type(&bean_type.1);

        let mutable_singleton_field_ids = Self::get_mutable_singleton_field_ids(bean);
        let singleton_field_ids = Self::get_singleton_field_ids(bean);

        let mutable_prototype_field_ids = Self::get_mutable_prototype_field_ids(bean);
        let prototype_field_ids = Self::get_prototype_field_ids(bean);

        let mutable_abstract_singleton_fields = Self::get_abstract_mutable_singleton_field_ids(bean);
        let mutable_abstract_prototype_fields = Self::get_abstract_mutable_prototype_field_ids(&bean);

        let abstract_singleton_fields = Self::get_abstract_singleton_field_ids(bean);
        let abstract_prototype_fields = Self::get_abstract_prototype_field_ids(bean);

        let default_field_info = Self::get_default_fields(&bean_type.0);

        vec![
            BeanFactoryInfo {
                singleton_field_type_info: BeansFieldTypeInfo {
                    concrete_field_type_info: singleton_field_ids,
                    concrete_mutable_field_type_info: mutable_singleton_field_ids,
                    abstract_field_type_info: abstract_singleton_fields,
                    abstract_mutable_field_type_info: mutable_abstract_singleton_fields,
                },
                prototype_field_type_info: BeansFieldTypeInfo {
                    concrete_field_type_info: prototype_field_ids,
                    concrete_mutable_field_type_info: mutable_prototype_field_ids,
                    abstract_field_type_info: abstract_prototype_fields,
                    abstract_mutable_field_type_info: mutable_abstract_prototype_fields,
                },
                default_field_info,
                concrete_type: bean.struct_type.clone(),
                abstract_type,
                is_enum: bean.enum_found.is_some(),
                ident_type: bean.ident.clone(),
                profile: Some(bean_type.2.to_owned()),
                factory_fn: bean.factory_fn.clone(),
                constructable: bean_type.0.is_constructable(),
                is_default: bean_type.0.has_default()
            }
        ]
    }
}