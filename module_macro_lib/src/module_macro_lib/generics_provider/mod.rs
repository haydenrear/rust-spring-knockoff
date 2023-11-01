use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{Item, WhereClause, WherePredicate};
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::impl_parse_values;
use module_macro_shared::item_modifier::ItemModifier;
use module_macro_shared::parse_container::{MetadataItem, MetadataItemId, ParseContainer, ParseContainerItemUpdater, ParseContainerModifier};
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;
use module_macro_shared::profile_tree::ProfileTree;
use crate::module_macro_lib::generics_provider::default_generics_provider::DefaultGenericsProvider;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;

pub mod struct_generics_provider;
pub mod impl_item_generics_provider;
pub mod function_generics_provider;
pub mod enum_generics_provider;
pub mod default_generics_provider;

/// There are several generics involved. However there is also whether or not it is a parameter
/// or it is a concrete struct/trait object, and within these we have whether or not
///
/// # types
/// concrete struct
/// trait object
/// enum
/// functions - these two have similar behavior
/// - method level
/// - static function level
/// # generic types
/// generic parameter
/// trait object
/// struct
/// # bounds
/// trait object bounds
/// generic parameter bounds
///
/// the question is - whether or not the framework needs to be aware of # generic types.
///  Obviously it needs to add impl <T> so yes. Moreover, they need to be merged together.
///  Knowing when to inject something where can also be based on the bounds.
///
/// The complexity mostly comes from trait object and the injection thereof.
///
/// The types passed to the BeanFactoryInfo need to include the generic params, and then which
/// generics are parameters needs to be distinguished and included so they can be included as impl <param>.
/// So then there is an addition of the generics passed to the factory, the addition of the generic params in the types. Morevoer,
/// there is an additional layer of abstraction introduced by the structs which have different
/// params. So, then the user either creates impls for the structs with particular types, or
/// else includes it in #[autowire(Qualifier)] on the field.
///
/// The BeanFactory is created at compile time. So this means that all of the values that satisfy
/// the bounds of the generics need to have a bean factory created. Either that or the user explicitly
/// specifies which bounds to include.
///
/// Currently the type of the BeanFactory is supposed to be the "concrete" type. However, a new
/// factory could be created which places a dyn in the type U = ... However this is dynamic dispatch.

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum TypeOrParam {
    Type, Param
}

#[derive(Default)]
pub struct GenericsResult {
    generic_type_params: HashMap<usize, Box<dyn ToTokens>>,
    generic_type_params_index: HashMap<usize, TypeOrParam>,
    where_clause_type_params_bounds: HashMap<usize, WherePredicate>,
    mutable_field_idents_ty: HashMap<usize, (Ident, Box<dyn ToTokens>)>,
    abstract_field_idents_ty: HashMap<usize, (Ident, Box<dyn ToTokens>)>,
    concrete_field_idents_ty: HashMap<usize, (Ident, Box<dyn ToTokens>)>,
    bean_factory_info: BeanFactoryInfo
}

impl Debug for GenericsResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

pub struct GenericsResultError {
    message: String
}

impl MetadataItem for GenericsResult {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl_parse_values!(GenericsResult);

pub enum GenericsResultType {
    Abstract, Concrete
}

impl GenericsResult {
    pub fn create_metadata_item_id(
        result_type: GenericsResultType,
        bean_id: &str
    ) -> MetadataItemId {
        return MetadataItemId {
            item_id: if matches!(result_type, GenericsResultType::Abstract) {
                format!("abstract_generics_result_{}", bean_id)
            } else {
                format!("concrete_generics_result_{}", bean_id)
            },
            metadata_item_type_id: "GenericsResult".to_string(),
        }
    }
}

/// All of the beans need to be in the container before the Generics TokenStream is set for
/// each of the BeanDefinitionTypes.
pub trait GenericsProvider: ProfileTreeModifier {
    fn provide_generics(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) -> Result<GenericsResult, GenericsResultError>;

}

pub struct DelegatingGenericsProvider {
    profile_tree_modifiers: Vec<Box<dyn GenericsProvider>>
}

impl ProfileTreeModifier for DelegatingGenericsProvider {
    fn modify_bean(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) {
        self.profile_tree_modifiers.iter().for_each(|m| m.modify_bean(dep_type, profile_tree));
    }

    fn new(profile_tree_items: &HashMap<String, BeanDefinition>) -> Self where Self: Sized {
        Self {
            profile_tree_modifiers: vec![Box::new(DefaultGenericsProvider::new(profile_tree_items)) as Box<dyn GenericsProvider>]
        }
    }
}

impl GenericsProvider for DelegatingGenericsProvider {
    fn provide_generics(&self, dep_type: &mut BeanDefinition, profile_tree: &mut ProfileTree) -> Result<GenericsResult, GenericsResultError> {
        Ok(GenericsResult {
            generic_type_params: Default::default(),
            generic_type_params_index: Default::default(),
            where_clause_type_params_bounds: Default::default(),
            mutable_field_idents_ty: Default::default(),
            abstract_field_idents_ty: Default::default(),
            concrete_field_idents_ty: Default::default(),
            bean_factory_info: Default::default(),
        })
    }
}