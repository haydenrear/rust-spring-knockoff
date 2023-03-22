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

pub struct MutableBeanFactoryGenerator {
    concrete_bean_factories: Vec<BeanFactoryInfo>,
    abstract_bean_factories: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for MutableBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

impl BeanFactoryGenerator for MutableBeanFactoryGenerator {

    fn concrete_bean_factory_tokens(concrete_type: &Ident, profile_ident: &Ident) -> TokenStream {
        quote! {
            impl MutableBeanFactory<Mutex<#concrete_type>, #profile_ident> for ListableBeanFactory {
                type U = Mutex<#concrete_type>;
                fn get_bean(&self) -> MutableBeanDefinition<Mutex<#concrete_type >> {
                    let this_component = <MutableBeanDefinition<Mutex<#concrete_type >>>::get_bean(&self);
                    this_component
                }
            }
        }
    }

    fn concrete_factory_bean(concrete_type: &Ident, profile_ident: &Ident, create_bean_tokens: TokenStream) -> TokenStream {
        quote! {
            impl MutableFactoryBean<Mutex<#concrete_type>, #profile_ident> for MutableBeanDefinition<Mutex<#concrete_type >> {
                type U = Mutex<#concrete_type>;
                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> MutableBeanDefinition<Mutex<#concrete_type >> {

                    #create_bean_tokens

                    Self {
                        inner: Arc::new(Mutex::new(inner))
                    }
                }

                fn get_bean_type_id(&self) -> TypeId {
                    self.inner.deref().type_id().clone()
                }

                fn is_singleton() -> bool {
                    true
                }

            }
        }
    }

    fn concrete_bean_container(concrete_type: &Ident, profile_ident: &Ident) -> TokenStream {
        quote! {
            impl BeanContainer<Mutex<#concrete_type >> for ListableBeanFactory {
                type U = Mutex<#concrete_type>;
                fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                    self.mutable_bean_definitions.get(&TypeId::of::<Arc<Mutex<#concrete_type >>>())
                        .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                        .flatten()
                }
            }

            impl BeanContainerProfile<Mutex<#concrete_type>, #profile_ident> for ListableBeanFactory {
                type U = Mutex<#concrete_type>;
                fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                    self.mutable_bean_definitions.get(&TypeId::of::<Arc<Mutex<#concrete_type >>>())
                        .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                        .flatten()
                }
            }
        }
    }

    fn abstract_bean_factory_tokens(concrete_type: &Ident, abstract_type: &Type, profile_ident: &Ident) -> TokenStream {
        quote! {
            impl MutableBeanFactory<Mutex<Box<dyn #abstract_type>>, #profile_ident> for ListableBeanFactory {
                type U = Mutex<Box<dyn #abstract_type>>;
                fn get_bean(&self) -> MutableBeanDefinition<Self::U> {
                    <MutableBeanDefinition<Mutex<Box<dyn #abstract_type>>>>::get_bean(&self)
                }
            }
        }
    }

    fn abstract_factory_bean(concrete_type: &Ident, profile_ident: &Ident, abstract_type: &Type, create_bean_tokens: TokenStream) -> TokenStream {
        quote! {
            impl MutableFactoryBean<Mutex<Box<dyn #abstract_type>>, #profile_ident> for MutableBeanDefinition<Mutex<Box<dyn #abstract_type>>> {
                type U = Mutex<Box<dyn #abstract_type>>;
                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> MutableBeanDefinition<Self::U> {

                    #create_bean_tokens

                    let m = MutableBeanDefinition {
                        inner: Arc::new(Mutex::new(Box::new(inner) as Box<dyn #abstract_type>))
                    };
                    m
                }

                fn get_bean_type_id(&self) -> TypeId {
                    self.inner.deref().type_id().clone()
                }

                fn is_singleton() -> bool {
                    true
                }

            }
        }
    }

    fn abstract_bean_container(concrete_type: &Ident, abstract_type: &Type, profile_ident: &Ident) -> TokenStream {
        quote! {
            impl BeanContainer<Mutex<Box<dyn #abstract_type>>> for ListableBeanFactory {
                type U = Mutex<Box<dyn #abstract_type>>;
                fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                    self.mutable_bean_definitions.get(&TypeId::of::<Arc<Mutex<Box<dyn #abstract_type>>>>())
                        .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                        .flatten()
                }
            }

            impl BeanContainerProfile<Mutex<Box<dyn #abstract_type>>, #profile_ident> for ListableBeanFactory {
                type U = Mutex<Box<dyn #abstract_type>>;
                fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                    self.mutable_bean_definitions.get(&TypeId::of::<Arc<Mutex<Box<dyn #abstract_type>>>>())
                        .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                        .flatten()
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
