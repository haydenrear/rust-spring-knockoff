use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Type;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::BeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;

use codegen_utils::syn_helper::SynHelper;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("factory_fn_factory_generator.rs");

pub struct FactoryFnBeanFactoryGenerator {
    concrete_bean_factories: Vec<BeanFactoryInfo>,
    abstract_bean_factories: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for FactoryFnBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

impl BeanFactoryGenerator for FactoryFnBeanFactoryGenerator {
    fn create_bean_tokens<ConcreteTypeT: ToTokens>(
        bean_factory_info: &BeanFactoryInfo,
        profile_ident: &Ident,
        concrete_type: &ConcreteTypeT
    ) -> Option<TokenStream> {
        /// Handled in the
        Self::create_bean_tokens_default(bean_factory_info, profile_ident, concrete_type)
    }

    fn create_bean_tokens_default<ConcreteTypeT: ToTokens>(
        bean_factory_info: &BeanFactoryInfo,
        profile_ident: &Ident,
        concrete_type: &ConcreteTypeT
    ) -> Option<TokenStream> {

        if bean_factory_info.factory_fn.is_none() {
            log_message!("Skipping creation of factory_fn for: {} as is not factory function.",
                SynHelper::get_str(concrete_type));
            return None;
        }

        let (field_idents, field_types,  concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_idents, abstract_mutable_field_types, concrete_mutable_abstract)
            = bean_factory_info.get_field_singleton_types();

        let (prototype_field_idents, prototype_field_types,  prototype_concrete_field,
            prototype_mutable_identifiers, prototype_mutable_field_types, prototype_concrete_mutable_type,
            prototype_abstract_field_idents, prototype_abstract_field_types, prototype_concrete_abstract_types,
            prototype_abstract_mutable_idents, prototype_abstract_mutable_field_types, prototype_concrete_mutable_abstract)
            = bean_factory_info.get_field_prototype_types();

        let (fn_args, factory_fn) = Self::get_factory_fn_fn_args(&bean_factory_info);

        log_message!("Creating factory for profile {} for factory_fn with ident {} and struct type: {}, and ident {}.",
            SynHelper::get_str(profile_ident),
            SynHelper::get_str(&bean_factory_info.factory_fn.as_ref().unwrap().fn_found.item_fn.sig.ident),
            SynHelper::get_str(&bean_factory_info.concrete_type.as_ref().unwrap()),
            SynHelper::get_str(&bean_factory_info.ident_type.as_ref().unwrap()),
        );

        log_message!("{} is number of field idents, {} is number of field types.", field_types.len(), field_idents.len());
        log_message!("{} is number of mutable idents, {} is number of mutable field types.", mutable_identifiers.len(), mutable_field_types.len());
        log_message!("{} is number of abstract idents, {} is number of abstract field types.", abstract_field_idents.len(), abstract_field_types.len());
        log_message!("{} is number of abstract mutable idents, {} is number of abstract mutable field types.", abstract_mutable_idents.len(), abstract_mutable_field_types.len());

        let create_beans_tokens = quote! {
                #(
                    let bean_def: BeanDefinition<#field_types> = <ListableBeanFactory as BeanFactory<#field_types, #profile_ident >>::get_bean(listable_bean_factory);
                    let #field_idents = bean_def.inner;
                )*
                #(
                    let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>>
                        = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>, #profile_ident >>::get_bean(listable_bean_factory);
                    let #mutable_identifiers = bean_def.inner;
                )*
                #(
                    let arc_bean_def = <ListableBeanFactory as BeanFactory<#abstract_field_types, #profile_ident >>::get_bean(listable_bean_factory);
                    let #abstract_field_idents = arc_bean_def.inner;
                )*
                #(
                    let bean_def = <ListableBeanFactory as MutableBeanFactory<Mutex<Box<#abstract_mutable_field_types>>, #profile_ident >>::get_bean(
                            listable_bean_factory
                        );
                    let #abstract_mutable_idents = bean_def.inner;
                )*

                /// Prototype beans
                #(
                    let bean_def: #prototype_field_types = < ListableBeanFactory as PrototypeBeanFactory<#prototype_field_types, #profile_ident> >::get_prototype_bean(listable_bean_factory);
                    let #prototype_field_idents = bean_def;
                )*
                #(
                    let bean_def: #prototype_mutable_field_types
                        = < ListableBeanFactory as PrototypeBeanFactory<#prototype_mutable_field_types, #profile_ident> >::get_prototype_bean(listable_bean_factory);
                    let #prototype_mutable_identifiers = Mutex::new(bean_def);
                )*
                #(
                    let arc_bean_def = < ListableBeanFactory as PrototypeBeanFactory<#prototype_abstract_field_types, #profile_ident> >::get_prototype_bean(listable_bean_factory);
                    let #prototype_abstract_field_idents = arc_bean_def;
                )*
                #(
                    let bean_def = < ListableBeanFactory as MutableBeanFactory<#prototype_abstract_mutable_field_types, #profile_ident> >::get_prototype_bean(
                            listable_bean_factory
                        );
                    let #prototype_abstract_mutable_idents = Mutex::new(bean_def);
                )*
                let inner = #factory_fn(
                    #(#fn_args,)*
                );
        };

        create_beans_tokens.into()
    }

    fn get_concrete_factories(&self) -> Vec<BeanFactoryInfo> {
        self.concrete_bean_factories.clone()
    }

    fn get_abstract_factories(&self) -> Vec<BeanFactoryInfo> {
        self.abstract_bean_factories.clone()
    }

    fn new(concrete_bean_factories: Vec<BeanFactoryInfo>, abstract_bean_factories: Vec<BeanFactoryInfo>) -> Self {
        Self {
            concrete_bean_factories,
            abstract_bean_factories,
        }
    }
}

impl FactoryFnBeanFactoryGenerator {
    fn get_factory_fn_fn_args(bean_factory_info: &BeanFactoryInfo) -> (Vec<&Ident>, &Ident) {
        let factory_fn = &bean_factory_info.factory_fn
            .as_ref()
            .expect("Factory function was not present in factory function bean factory generator!")
            .fn_found;
        let fn_args = factory_fn.args.iter()
            .map(|a| &a.0)
            .collect::<Vec<&Ident>>();
        let factory_fn = &factory_fn.item_fn.sig.ident;
        (fn_args, factory_fn)
    }
}
