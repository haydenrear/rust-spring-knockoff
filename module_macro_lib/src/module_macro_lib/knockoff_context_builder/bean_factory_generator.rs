use std::rc::Rc;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::token::Mut;
use syn::{Path, Type};
use codegen_utils::syn_helper::SynHelper;
use factory_factory_generator::FactoryBeanBeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use module_macro_shared::bean::{BeanDefinition, BeanPath};
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::{AbstractBeanFactoryInfo, BeanFactoryInfo, BeanFactoryInfoFactory, ConcreteBeanFactoryInfo};

use module_macro_shared::dependency::{AutowiredField, DependencyDescriptor, FieldDepType};
use module_macro_shared::profile_tree::ProfileBuilder;
use mutable_factory_generator::MutableBeanFactoryGenerator;
use prototype_factory_generator::PrototypeBeanFactoryGenerator;

use knockoff_logging::{initialize_log, use_logging};
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::factory_fn_factory_generator::FactoryFnBeanFactoryGenerator;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub mod mutable_factory_generator;
pub mod prototype_factory_generator;
pub mod factory_factory_generator;
pub mod factory_fn_factory_generator;

pub trait BeanFactoryGenerator: TokenStreamGenerator {

    fn concrete_bean_factory_tokens(concrete_type: &Ident, profile_ident: &Ident) -> TokenStream {
        quote! {
            impl BeanFactory<#concrete_type, #profile_ident> for ListableBeanFactory {
                type U = #concrete_type;
                fn get_bean(&self) -> BeanDefinition<#concrete_type> {
                    let this_component = <BeanDefinition<#concrete_type>>::get_bean(&self);
                    this_component
                }
            }
        }
    }

    fn concrete_factory_bean(concrete_type: &Ident, profile_ident: &Ident, create_bean_tokens: TokenStream) -> TokenStream {
        quote! {
                impl FactoryBean<#concrete_type, #profile_ident> for BeanDefinition<#concrete_type> {
                    type U = #concrete_type;
                    fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<#concrete_type> {

                        #create_bean_tokens

                        Self {
                            inner: Arc::new(inner)
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
                impl BeanContainer<#concrete_type> for ListableBeanFactory {
                    type U = #concrete_type;
                    fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                        self.singleton_bean_definitions.get(&TypeId::of::<Arc<#concrete_type>>())
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

                impl BeanContainerProfile<#concrete_type, #profile_ident> for ListableBeanFactory {
                    type U = #concrete_type;
                    fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                        self.singleton_bean_definitions.get(&TypeId::of::<Arc<#concrete_type>>())
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }
        }
    }

    fn abstract_bean_factory_tokens(concrete_type: &Ident, abstract_type: &Type, profile_ident: &Ident) -> TokenStream {
        quote! {
            impl BeanFactory<dyn #abstract_type, #profile_ident> for ListableBeanFactory {
                type U = #concrete_type;
                fn get_bean(&self) -> BeanDefinition<#concrete_type> {
                   let bean_def: BeanDefinition<#concrete_type> = <BeanDefinition<dyn #abstract_type> as FactoryBean<dyn #abstract_type, #profile_ident>>::get_bean(&self);
                    bean_def
                }
            }
        }
    }

    fn abstract_factory_bean(concrete_type: &Ident, profile_ident: &Ident, abstract_type: &Type, create_bean_tokens: TokenStream) -> TokenStream {
        quote! {
            impl FactoryBean<dyn #abstract_type, #profile_ident> for BeanDefinition<dyn #abstract_type> {
                type U = #concrete_type;

                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<#concrete_type> {

                    #create_bean_tokens

                    BeanDefinition {
                        inner: Arc::new(inner)
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

    fn abstract_bean_container(concrete_type: &Ident, abstract_type: &Type, profile_ident: &Ident) -> TokenStream {
        quote! {
            impl BeanContainer<dyn #abstract_type> for ListableBeanFactory {
                type U = #concrete_type;
                fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                    let type_id = TypeId::of::<Arc<dyn #abstract_type>>();
                    self.singleton_bean_definitions.get(&type_id)
                        .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                        .flatten()
                }
            }

            impl BeanContainerProfile<dyn #abstract_type, #profile_ident> for ListableBeanFactory {
                type U = #concrete_type;
                fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                    let type_id = TypeId::of::<Arc<dyn #abstract_type>>();
                    self.singleton_bean_definitions.get(&type_id)
                        .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                        .flatten()
                }
            }
        }
    }

    fn create_concrete_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream {

        let profile_ident = &bean_factory_info.get_profile_ident();
        let concrete_type = bean_factory_info.get_concrete_type();

        Self::create_bean_tokens(bean_factory_info, profile_ident, &concrete_type)
            .map(|create_bean_tokens| {
                let bean_factory = Self::concrete_bean_factory_tokens(&concrete_type, profile_ident);
                let bean_container = Self::concrete_bean_container(&concrete_type, profile_ident);
                let factory_bean = Self::concrete_factory_bean(&concrete_type, profile_ident, create_bean_tokens);

                let injectable_code = quote! {
                    #bean_factory
                    #bean_container
                    #factory_bean
                };
                injectable_code.into()
            })
            .or(Some(TokenStream::default()))
            .unwrap()

    }

    fn create_abstract_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    )  -> TokenStream {


        log_message!("Building factory generator for {}", SynHelper::get_str(&bean_factory_info.abstract_type.as_ref().unwrap()));

        let abstract_type: &Type = bean_factory_info.abstract_type.as_ref().unwrap();

        let profile_ident = &bean_factory_info.get_profile_ident();
        let concrete_type = bean_factory_info.get_concrete_type();

        Self::create_bean_tokens(bean_factory_info, profile_ident, &concrete_type)
            .map(|create_bean_tokens| {

                let bean_factory = Self::abstract_bean_factory_tokens(&concrete_type, abstract_type, profile_ident);
                let bean_container = Self::abstract_bean_container(&concrete_type, abstract_type, profile_ident);
                let factory_bean = Self::abstract_factory_bean(&concrete_type, profile_ident, abstract_type, create_bean_tokens);

                let injectable_code = quote! {
                    #bean_factory
                    #bean_container
                    #factory_bean
                };

                injectable_code.into()
            })
            .or(Some(TokenStream::default()))
            .unwrap()
    }

    fn new_bean_factory_generators(concrete_beans: &Vec<BeanFactoryInfo>, abstract_beans: &Vec<BeanFactoryInfo>) -> Vec<Box<dyn TokenStreamGenerator>> {
        vec![
            Box::new(MutableBeanFactoryGenerator::new_bean_factory_generator(concrete_beans.clone(), abstract_beans.clone())) as Box<dyn TokenStreamGenerator>,
            Box::new(FactoryBeanBeanFactoryGenerator::new_bean_factory_generator(concrete_beans.clone(), abstract_beans.clone())) as Box<dyn TokenStreamGenerator>,
            Box::new(PrototypeBeanFactoryGenerator::new_bean_factory_generator(concrete_beans.clone(), abstract_beans.clone())) as Box<dyn TokenStreamGenerator>,
            Box::new(FactoryFnBeanFactoryGenerator::new_bean_factory_generator(concrete_beans.clone(), abstract_beans.clone())) as Box<dyn TokenStreamGenerator>
        ]
    }

    fn generate_factories(&self) -> TokenStream {
        let mut ts = TokenStream::default();

        self.get_concrete_factories().iter()
            .for_each(|b| {
                ts.append_all(Self::create_concrete_bean_factories_for_bean(b));
            });

        self.get_abstract_factories().iter()
            .for_each(|b| {
                if !b.abstract_type.is_none() {
                    if b.ident_type.is_some() {
                        ts.append_all(Self::create_abstract_bean_factories_for_bean(b));
                    } else if b.concrete_type.is_some() {
                        ts.append_all(Self::create_abstract_bean_factories_for_bean(b));
                    }
                }
            });

        ts
    }

    fn create_bean_tokens(
        bean_factory_info: &BeanFactoryInfo,
        profile_ident: &Ident,
        concrete_type: &Ident
    ) -> Option<TokenStream> {

        if bean_factory_info.factory_fn.is_some() {
            log_message!("Skipping creation of bean factory for {} because has factory fn.", SynHelper::get_str(&bean_factory_info.concrete_type.as_ref().unwrap()));
            return None;
        }

        let (field_types, field_idents, concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_idents, abstract_mutable_field_types, concrete_mutable_abstract)
            = bean_factory_info.get_field_types();

        log_message!("Creating factory for profile {} for: {}.",
            SynHelper::get_str(profile_ident),
            SynHelper::get_str(&bean_factory_info.concrete_type.as_ref().unwrap())
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
                let inner = #concrete_type::new(
                    #(#field_idents,)* #(#mutable_identifiers,)*
                    #(#abstract_field_idents,)* #(#abstract_mutable_idents,)*
                );
        };

        create_beans_tokens.into()
    }

    fn new_bean_factory_generator(concrete_beans: Vec<BeanFactoryInfo>, abstract_beans: Vec<BeanFactoryInfo>) -> Self
        where Self: Sized
    {
        Self::new(concrete_beans, abstract_beans)
    }

    fn get_concrete_factories(&self) -> Vec<BeanFactoryInfo>;

    fn get_abstract_factories(&self) -> Vec<BeanFactoryInfo>;

    fn new(beans: Vec<BeanFactoryInfo>, abstract_beans: Vec<BeanFactoryInfo>) -> Self;
}
