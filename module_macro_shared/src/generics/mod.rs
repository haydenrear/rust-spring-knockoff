use std::collections::HashMap;
use syn::{Generics, ImplGenerics, Type, TypeGenerics, WhereClause};
use proc_macro2::{Ident, TokenStream};
use crate::bean::{BeanDefinitionType, BeanType};
use crate::generics::Gens::ParsedGenericParts;
use crate::impl_parse_values;
use crate::parse_container::MetadataItemId;
use crate::profile_tree::ProfileTree;
use crate::bean::AbstractionLevel;


pub enum Gens {
    HeadGen {
        /// The whole shebang from this point forward, contains all sub-types, so if S<T<U>> then S<T<U>>
        gen_type_path: syn::Path,
        /// Provides the identifier for the head, so if it's S<T<U>>, then will be S
        head_ident: Option<Ident>,
        /// Provides the identifier for the head, so if it's dyn S<T<U>>, then will be dyn S
        abstract_type: Option<Type>,
        /// The type bounds
        type_bounds: Vec<Type>,
        /// To tokens provides the prefix when calling methods within the context of where the generic
        /// parameters are defined. Method calls are always called with the generics included. This
        /// is for when calling any method, not for passing generics to generic methods. Same for
        /// all methods called on a struct or impl, because the generic types are same.
        method_call_struct_prefix: TokenStream,
    },
    ParsedGenericParts {
        /// The whole shebang from this point forward, contains all sub-types, so if S<T<U>> then S<T<U>>
        gen_type_path: syn::Path,
        /// Provides the identifier only, so if it's S<T<U>>, then will be S
        gen_ident: Option<Ident>,
        /// Provides the identifier for the head, so if it's dyn S<T<U>>, then will be dyn S
        abstract_ident: Option<Type>,
        /// The type bounds
        type_bounds: Vec<Type>,
    }
}

pub enum GenericsDefinitionType {
    Trait, StructOrEnumTraitImpl, StructOrEnum,
    StructOrEnumSelfImpl, StructOrEnumFieldOrArg,
}


/// Exists for each field, struct, impl.
pub struct GenericsResult {
    head_gen: Gens,
    generic_parameters: Vec<Gens>,
    type_parameters: Vec<Gens>,
    generics: Generics,
    defn_type: GenericsDefinitionType
}



impl GenericsResult {
    pub fn create_metadata_item_id(
        result_type: BeanType,
        bean_id: &str
    ) -> MetadataItemId {
        return MetadataItemId {
            item_id: if matches!(&result_type, BeanType::Prototype(AbstractionLevel::Abstract))
                || matches!(&result_type, BeanType::Singleton(AbstractionLevel::Abstract)) {
                format!("abstract_generics_result_{}", bean_id)
            } else {
                format!("concrete_generics_result_{}", bean_id)
            },
            metadata_item_type_id: "GenericsResult".to_string(),
        }
    }
}

use crate::parse_container::MetadataItem;
impl_parse_values!(GenericsResult);

impl GenericsResult {

    /// Takes in a number of definitions, for example a struct, with it's generics, and a trait
    /// with it's generics and produces the generics for definition of the type for the macro.
    /// For example, if creating a bean factory for a particular implementation of a particular trait,
    /// then dyn Trait<G, T> will be mapped to Struct<T, U>, or some such, where the impl<G, T, U>
    /// will be an aggregate of the params for Struct and Trait. So then you need to merge the
    /// Generic definitions together for the impl<G, T, U> block from the two. This covers the case
    /// when that particular impl is not provided by the user, but the bounds are otherwise covered
    /// by some other impl. In this case there will be two BeanFactories mapping to the same
    /// bean.
    /// So if I have struct impl'ing Trait<G, F> and F: T, then need to create bean factory for
    /// Trait<G, F> that maps to the same bean that Trait<G, T> maps to.
    pub fn to_merged_defn_generics<'a>(
        in_results: Vec<GenericsResult>
    ) -> (ImplGenerics<'a>, TypeGenerics<'a>, Option<&'a WhereClause>) {
        todo!()
    }

    pub fn to_defn_generics<'a>(&self) -> (ImplGenerics<'a>, TypeGenerics<'a>, Option<&'a WhereClause>) {
        todo!()
    }

    pub fn to_method_call_generics<'a>(&self) -> TokenStream {
        todo!()
    }

    /// Each bean has multiple deps. So then for each bean dependency for that bean, parse the
    /// generics, so that the bean will be called correctly.
    pub fn parse_generics_for_bean_dep(
        bean_definition: &BeanDefinitionType,
        referring_def: &BeanDefinitionType,
        profile_tree: &mut ProfileTree
    ) -> Self {

        // GenericsResult {
        //     head_gen: Gens::HeadGen {
        //         gen_type_path: ,
        //         head_ident: None,
        //         abstract_type: None,
        //         type_bounds: vec![],
        //         method_call_prefix: Default::default(),
        //     },
        //     generic_parameters: vec![],
        //     type_parameters: vec![],
        //     gens: Default::default(),
        // }
        todo!()
    }

    /// Each bean is defined with some generics, so then define that bean and it's generics.
    pub fn parse_generics_for_self_bean(
        bean_definition: &BeanDefinitionType,
        profile_tree: &mut ProfileTree
    ) -> Self {
        todo!()
    }

    /// Each bean can be defined with some number of impls related to the type bounds on it's
    /// generics. These are required to be enumerated to create the factories, so that concrete
    /// beans can be created for all of the type bounds. So then the dynamic call to a dynamic
    /// bean factory produces a concrete type, resulting in no dynamic dispatch, with the benefits
    /// of dyn.
    pub fn parse_generics_for_self_bean_impl(

    ) -> Self {
        todo!()
    }

    /// Each bean is defined with some number of impls defined by traits that it explicitly impls.
    /// So parse the generics for those impls.
    pub fn parse_generics_for_trait_impl(

    ) -> Self {
        todo!()
    }
}