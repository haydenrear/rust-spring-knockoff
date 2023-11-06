use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::Type;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::BeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::factory_fn_factory_generator::FactoryFnBeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::mutable_factory_generator::MutableBeanFactoryGenerator;
import_logger!("prototype_factory_generator.rs");

pub struct PrototypeBeanFactoryGenerator {
    concrete_bean_factories: Vec<BeanFactoryInfo>,
    abstract_bean_factories: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for PrototypeBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

/// MutablePrototypeBeanFactoryGenerator not necessary because they'll prototype bean.
impl BeanFactoryGenerator for PrototypeBeanFactoryGenerator {

    fn concrete_bean_factory_tokens<ConcreteTypeT: ToTokens>(concrete_type: &ConcreteTypeT, profile_ident: &Ident) -> TokenStream {
        quote! {
        }
    }

    fn concrete_factory_bean<ConcreteTypeT: ToTokens>(concrete_type: &ConcreteTypeT,
                                                      profile_ident: &Ident,
                                                      create_bean_tokens: TokenStream) -> TokenStream {
        quote! {
            impl PrototypeBeanFactory<#concrete_type, #profile_ident> for ListableBeanFactory {
                type U = #concrete_type;

                fn get_prototype_bean(listable_bean_factory: &ListableBeanFactory) -> #concrete_type {

                    #create_bean_tokens

                    inner
                }

            }
        }
    }

    fn concrete_bean_container<ConcreteTypeT: ToTokens>(concrete_type: &ConcreteTypeT, profile_ident: &Ident) -> TokenStream {
        quote! {
            impl PrototypeBeanContainer<#concrete_type> for ListableBeanFactory {
                type U = #concrete_type;
                fn fetch_bean(&self) -> Self::U {
                    <ListableBeanFactory as PrototypeBeanFactory<#concrete_type, DefaultProfile>>::get_prototype_bean(&self)
                }
            }

            impl PrototypeBeanContainerProfile<#concrete_type, #profile_ident> for ListableBeanFactory {
                type U = #concrete_type;
                fn fetch_bean_profile(&self) -> Self::U {
                    <ListableBeanFactory as PrototypeBeanFactory<#concrete_type, #profile_ident>>::get_prototype_bean(&self)
                }
            }
        }
    }

    fn abstract_bean_factory_tokens<ConcreteTypeT: ToTokens, AbstractTypeT: ToTokens>(
        concrete_type: &ConcreteTypeT, abstract_type: &AbstractTypeT, profile_ident: &Ident) -> TokenStream {
        quote! {
        }
    }

    fn abstract_factory_bean<ConcreteTypeT: ToTokens, AbstractTypeT: ToTokens>(
        concrete_type: &ConcreteTypeT, profile_ident: &Ident, abstract_type: &AbstractTypeT,
        create_bean_tokens: TokenStream
    ) -> TokenStream {
        quote! {
            impl PrototypeBeanFactory<dyn #abstract_type, #profile_ident> for ListableBeanFactory {
                type U = #concrete_type;

                fn get_prototype_bean(listable_bean_factory: &ListableBeanFactory) -> #concrete_type {

                    #create_bean_tokens

                    inner
                }

            }
        }
    }

    fn abstract_bean_container<ConcreteTypeT: ToTokens, AbstractTypeT: ToTokens>(
        concrete_type: &ConcreteTypeT, abstract_type: &AbstractTypeT,
        profile_ident: &Ident
    ) -> TokenStream {
        quote! {
            impl PrototypeBeanContainer<dyn #abstract_type> for ListableBeanFactory {
                type U = #concrete_type;
                fn fetch_bean(&self) -> Self::U {
                    <ListableBeanFactory as PrototypeBeanFactory<#concrete_type, DefaultProfile>>::get_prototype_bean(&self)
                }
            }

            impl PrototypeBeanContainerProfile<dyn #abstract_type, #profile_ident> for ListableBeanFactory {
                type U = #concrete_type;
                fn fetch_bean_profile(&self) -> Self::U {
                    <ListableBeanFactory as PrototypeBeanFactory<#concrete_type, #profile_ident>>::get_prototype_bean(&self)
                }
            }
        }
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
