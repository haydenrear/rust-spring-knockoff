use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Type;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::BeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;

use codegen_utils::syn_helper::SynHelper;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

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

    fn create_bean_tokens(
        bean_factory_info: &BeanFactoryInfo,
        profile_ident: &Ident,
        concrete_type: &Ident
    ) -> Option<TokenStream> {

        if bean_factory_info.factory_fn.is_none() {
            log_message!("Skipping creation of factory_fn for: {}.", SynHelper::get_str(concrete_type));
            return None;
        }

        let (field_types, field_idents, concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_idents, abstract_mutable_field_types, concrete_mutable_abstract)
            = bean_factory_info.get_field_types();

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
