use std::rc::Rc;
use std::sync::Arc;
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{Field, Fields, Path, Type};
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::Bean;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::dependency::{AutowiredField, AutowireType, DepType};
use module_macro_shared::profile_tree::ProfileBuilder;
use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;


#[derive(Clone, Default)]
pub struct BeanFactoryInfo {
    pub(crate) fields: Vec<AutowirableFieldTypeInfo>,
    pub(crate) mutable_fields: Vec<MutableFieldInfo>,
    pub(crate) abstract_fields: Vec<AbstractFieldInfo>,
    pub(crate) mutable_abstract_fields: Vec<MutableAbstractFieldInfo>,
    pub(crate) default_field_info: Vec<DefaultFieldInfo>,
    pub(crate) concrete_type: Option<Type>,
    pub(crate) abstract_type: Option<Path>,
    pub(crate) ident_type: Option<Ident>,
    pub(crate) profile: Option<ProfileBuilder>
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
    autowire_type: AutowiredField,
    qualifier: Option<String>,
    profile: Option<String>
}

#[derive(Clone)]
pub struct MutableAbstractFieldInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident,
    autowire_type: AutowiredField,
    qualifier: Option<String>,
    profile: Option<String>
}

impl BeanFactoryInfo {

    pub(crate) fn get_profile_ident(&self) -> Ident {
        self.profile.as_ref().map(|p| Ident::new(p.profile.as_str(), Span::call_site()))
            .or(Some(Ident::new(ProfileBuilder::default().profile.as_str(), Span::call_site())))
            .unwrap()
    }

    pub(crate) fn get_concrete_type(&self) -> Ident {
        self.concrete_type
            .as_ref()
            .map(|t| Ident::new(t.to_token_stream().to_string().as_str(), Span::call_site()))
            .or(self.ident_type.as_ref().map(|i| i.clone()))
            .unwrap()
    }

    pub(crate) fn get_field_types(&self)
                                  -> (Vec<Type>, Vec<Ident>,
                                      Vec<Type>, Vec<Ident>,
                                      Vec<Type>, Vec<Type>,
                                      Vec<Ident>, Vec<Type>,
                                      Vec<Type>, Vec<Ident>,
                                      Vec<Type>, Vec<Type>) {

        let field_types = self.fields.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let field_idents = self.fields.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let field_concrete = self.fields.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_field_idents = self.mutable_fields.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let mutable_field_types = self.mutable_fields.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_field_concrete = self.mutable_fields.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let abstract_field_ident = self.abstract_fields.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let abstract_field_types = self.abstract_fields.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let abstract_field_concrete = self.abstract_fields.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_abstract_field_ident = self.mutable_abstract_fields.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let mutable_abstract_field_types = self.mutable_abstract_fields.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_abstract_field_concrete = self.mutable_abstract_fields.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        (field_types, field_idents, field_concrete,
         mutable_field_idents, mutable_field_types, mutable_field_concrete,
         abstract_field_ident, abstract_field_types, abstract_field_concrete,
         mutable_abstract_field_ident, mutable_abstract_field_types, mutable_abstract_field_concrete)
    }
}

pub trait BeanFactoryInfoFactory<T> {

    fn create_bean_factory_info(bean: &T) -> Vec<BeanFactoryInfo>;

    fn get_mutable_singleton_field_ids(token_type: &Bean) -> Vec<MutableFieldInfo> {
        Self::get_field_ids::<MutableFieldInfo>(token_type, &Self::create_mutable_dep_type)
    }

    fn get_singleton_field_ids(bean: &Bean) -> Vec<AutowirableFieldTypeInfo> {
        Self::get_field_ids::<AutowirableFieldTypeInfo>(bean, &Self::create_dep_type)
    }

    fn get_abstract_field_ids(bean: &Bean) -> Vec<AbstractFieldInfo> {
        Self::get_field_ids::<AbstractFieldInfo>(bean, &Self::create_abstract_dep_type)
    }

    fn get_abstract_mutable_field_ids(bean: &Bean) -> Vec<MutableAbstractFieldInfo> {
        Self::get_field_ids::<MutableAbstractFieldInfo>(
            bean,
            &Self::create_mutable_abstract_dep_type
        )
    }

    fn get_default_fields(bean: &Bean) -> Vec<DefaultFieldInfo> {
        if bean.fields.len() > 1 {
            panic!("Type had more than one set of fields Enum is not ready to be autowired!");
        }
        match &bean.fields[0] {
            Fields::Named(n) => {
                n.named.iter()
                    .filter(|f| SynHelper::get_attr_from_vec(&f.attrs, vec!["autowired"]).is_none())
                    .map(|f| DefaultFieldInfo{ field_type: f.ty.clone(), field_ident: f.ident.as_ref().unwrap().clone() } )
                    .collect::<Vec<DefaultFieldInfo>>()
            }
            _ => {
                vec![]
            }
        }
    }

    fn create_mutable_dep_type(dep_type: &DepType) -> Option<MutableFieldInfo> {
        if dep_type.is_abstract.is_some() && *dep_type.is_abstract.as_ref().unwrap() {
            return None;
        }
        dep_type.bean_type_path
            .as_ref()
            .filter(|d| d.is_mutable())
            .map(|type_path| type_path.get_autowirable_type())
            .flatten()
            .map(|field_type| MutableFieldInfo {
                concrete_field_type: dep_type.bean_info.concrete_type_of_field_bean_type.clone().or(Some(field_type.clone())).unwrap(),
                field_type,
                field_ident: dep_type.bean_info.field.ident.clone().unwrap(),
            })
    }

    fn create_dep_type(dep_type: &DepType) -> Option<AutowirableFieldTypeInfo> {
        if dep_type.is_abstract.is_some() && *dep_type.is_abstract.as_ref().unwrap() {
            return None;
        }
        dep_type.bean_type_path
            .as_ref()
            .filter(|d| d.is_not_mutable())
            .map(|type_path| type_path.get_autowirable_type())
            .flatten()
            .map(|field_type| AutowirableFieldTypeInfo {
                concrete_field_type: dep_type.bean_info.concrete_type_of_field_bean_type.clone().or(Some(field_type.clone())).unwrap(),
                field_type,
                field_ident: dep_type.bean_info.field.ident.clone().unwrap(),
            })
    }

    fn create_abstract_dep_type(dep_type: &DepType) -> Option<AbstractFieldInfo> {
        if dep_type.is_abstract.is_some() && *dep_type.is_abstract.as_ref().unwrap() {
            return dep_type.bean_type_path
                .as_ref()
                .filter(|d| d.is_not_mutable())
                .map(|type_path| type_path.get_autowirable_type())
                .flatten()
                .map(|field_type| AbstractFieldInfo {
                    concrete_field_type: dep_type.bean_info.concrete_type_of_field_bean_type.clone().or(Some(field_type.clone())).unwrap(),
                    field_type,
                    autowire_type: dep_type.bean_info.clone(),
                    qualifier: dep_type.bean_info.qualifier.clone(),
                    profile: None,
                    field_ident: dep_type.bean_info.field.ident.clone().unwrap(),
                });
        }
        None
    }

    fn create_mutable_abstract_dep_type(dep_type: &DepType) -> Option<MutableAbstractFieldInfo> {
        if dep_type.is_abstract.is_some() && *dep_type.is_abstract.as_ref().unwrap() {
            return dep_type.bean_type_path
                .as_ref()
                .filter(|d| d.is_mutable())
                .map(|type_path| type_path.get_autowirable_type())
                .flatten()
                .map(|field_type| MutableAbstractFieldInfo {
                    concrete_field_type: dep_type.bean_info.concrete_type_of_field_bean_type.clone().or(Some(field_type.clone())).unwrap(),
                    field_type,
                    autowire_type: dep_type.bean_info.clone(),
                    qualifier: dep_type.bean_info.qualifier.clone(),
                    profile: None,
                    field_ident: dep_type.bean_info.field.ident.clone().unwrap(),
                });
        }
        None
    }

    fn get_field_ids<U>(
        token_type: &Bean,
        creator: &dyn Fn(&DepType) -> Option<U>
    ) -> Vec<U> {
        let field_types = token_type.deps_map
            .clone().iter()
            .flat_map(|d| creator(d)
                .map(|item| vec![item])
                .or(Some(vec![]))
                .unwrap()
            )
            .collect::<Vec<U>>();

        field_types
    }

    fn bean_dep_impl_not_abstract(b: &DepType) -> bool {
        b.is_abstract.is_some() && !*b.is_abstract.as_ref().unwrap()
    }

    fn bean_dep_impl_abstract(b: &DepType) -> bool {
        !Self::bean_dep_impl_not_abstract(b)
    }

}

/// ConcreteBeanFactoryInfo is the default implementation of the beans... So the Profile is only used
/// when there is an abstract bean that will be different for different Profiles or qualifiers.
pub struct ConcreteBeanFactoryInfo;

impl BeanFactoryInfoFactory<Bean> for ConcreteBeanFactoryInfo {

    fn create_bean_factory_info(bean: &Bean) -> Vec<BeanFactoryInfo> {
        log_message!("Creating bean factory info for bean with id {} with has {} dependencies.", &bean.id, bean.deps_map.len());

        // TODO: Fix this - not adding dependencies in some cases.
        let mutable_fields = Self::get_mutable_singleton_field_ids(bean);
        let fields = Self::get_singleton_field_ids(bean);
        let mutable_abstract_fields = Self::get_abstract_mutable_field_ids(&bean);
        let abstract_fields = Self::get_abstract_field_ids(&bean);
        let default_field_info = Self::get_default_fields(&bean);

        bean.profile.iter()
            .map(|p| BeanFactoryInfo {
                fields: fields.clone(),
                mutable_fields: mutable_fields.clone(),
                abstract_fields: abstract_fields.clone(),
                mutable_abstract_fields: mutable_abstract_fields.clone(),
                default_field_info: default_field_info.clone(),
                concrete_type: bean.struct_type.clone(),
                abstract_type: None,
                ident_type: bean.ident.clone(),
                profile: Some(ProfileBuilder::default()),
            })
            .collect::<Vec<BeanFactoryInfo>>()
    }
}

pub struct AbstractBeanFactoryInfo;

impl BeanFactoryInfoFactory<(Bean, AutowireType, ProfileBuilder)> for AbstractBeanFactoryInfo {
    fn create_bean_factory_info(bean_type: &(Bean, AutowireType, ProfileBuilder)) -> Vec<BeanFactoryInfo> {

        let bean = bean_type.0.to_owned();
        let abstract_type = bean_type.1.to_owned().item_impl.trait_.map(|t| t.1);
        let mutable_fields = Self::get_mutable_singleton_field_ids(&bean);
        let fields = Self::get_singleton_field_ids(&bean);
        let mutable_abstract_fields = Self::get_abstract_mutable_field_ids(&bean);
        let abstract_fields = Self::get_abstract_field_ids(&bean);
        let default_field_info = Self::get_default_fields(&bean_type.0);

        vec![
            BeanFactoryInfo {
                fields,
                mutable_fields,
                abstract_fields,
                mutable_abstract_fields,
                default_field_info,
                concrete_type: bean.struct_type.clone(),
                abstract_type,
                ident_type: bean.ident.clone(),
                profile: Some(bean_type.2.to_owned())
            }
        ]
    }
}