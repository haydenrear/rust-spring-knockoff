use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Type;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::BeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

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

    fn concrete_bean_factory_tokens(concrete_type: &Ident, profile_ident: &Ident) -> TokenStream {
        quote! {
        }
    }

    fn concrete_factory_bean(concrete_type: &Ident, profile_ident: &Ident, create_bean_tokens: TokenStream) -> TokenStream {
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

    fn concrete_bean_container(concrete_type: &Ident, profile_ident: &Ident) -> TokenStream {
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

    fn abstract_bean_factory_tokens(concrete_type: &Ident, abstract_type: &Type, profile_ident: &Ident) -> TokenStream {
        quote! {
        }
    }

    fn abstract_factory_bean(concrete_type: &Ident, profile_ident: &Ident, abstract_type: &Type, create_bean_tokens: TokenStream) -> TokenStream {
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

    fn abstract_bean_container(concrete_type: &Ident, abstract_type: &Type, profile_ident: &Ident) -> TokenStream {
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