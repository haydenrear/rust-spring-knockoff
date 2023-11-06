use std::rc::Rc;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::token::Mut;
use syn::{Path, Type};
use codegen_utils::syn_helper::SynHelper;
use factory_factory_generator::FactoryBeanBeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use module_macro_shared::bean::{BeanDefinition, BeanPath};
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::{AbstractBeanFactoryInfo, BeanFactoryInfo, ConcreteBeanFactoryInfo};

use module_macro_shared::profile_tree::ProfileBuilder;
use mutable_factory_generator::MutableBeanFactoryGenerator;
use prototype_factory_generator::PrototypeBeanFactoryGenerator;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::factory_fn_factory_generator::FactoryFnBeanFactoryGenerator;
import_logger!("bean_factory_generator.rs");


pub mod mutable_factory_generator;
pub mod prototype_factory_generator;
pub mod factory_factory_generator;
pub mod factory_fn_factory_generator;

pub trait BeanFactoryGenerator: TokenStreamGenerator {

    fn concrete_bean_factory_tokens<ConcreteTypeT: ToTokens>(concrete_type: &ConcreteTypeT,
                                                             profile_ident: &Ident) -> TokenStream {
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

    fn concrete_factory_bean<ConcreteTypeT: ToTokens>(concrete_type: &ConcreteTypeT,
                                                      profile_ident: &Ident,
                                                      create_bean_tokens: TokenStream) -> TokenStream {
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

    fn concrete_bean_container<ConcreteTypeT: ToTokens>(concrete_type: &ConcreteTypeT, profile_ident: &Ident) -> TokenStream {
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

    fn abstract_bean_factory_tokens<ConcreteTypeT: ToTokens, AbstractTypeT: ToTokens>(
        concrete_type: &ConcreteTypeT, abstract_type: &AbstractTypeT, profile_ident: &Ident) -> TokenStream {
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

    fn abstract_factory_bean<ConcreteTypeT: ToTokens, AbstractTypeT: ToTokens>(
        concrete_type: &ConcreteTypeT, profile_ident: &Ident, abstract_type: &AbstractTypeT,
        create_bean_tokens: TokenStream
    ) -> TokenStream {
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

    fn abstract_bean_container<ConcreteTypeT: ToTokens, AbstractTypeT: ToTokens>(
        concrete_type: &ConcreteTypeT, abstract_type: &AbstractTypeT,
        profile_ident: &Ident
    ) -> TokenStream {
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

        info!("Creating concrete factories for bean.");

        Self::create_bean_tokens(bean_factory_info, profile_ident, &concrete_type)
            .map(|create_bean_tokens| {
                info!("Creating concrete bean factory tokens.");
                let bean_factory = Self::concrete_bean_factory_tokens(&concrete_type, profile_ident);
                info!("Creating concrete bean container.");
                let bean_container = Self::concrete_bean_container(&concrete_type, profile_ident);
                info!("Creating concrete factory bean.");
                let factory_bean = Self::concrete_factory_bean(&concrete_type, profile_ident, create_bean_tokens);

                let injectable_code = quote! {
                    #bean_factory
                    #bean_container
                    #factory_bean
                };
                info!("Finished creating injectable code.");
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
        let ident_type = bean_factory_info.get_concrete_type_as_ident();

        info!("Creating abstract bean factories.");

        Self::create_bean_tokens(bean_factory_info, profile_ident, &concrete_type)
            .map(|create_bean_tokens| {

                info!("Creating abstract bean factory tokens.");
                if concrete_type.is_some() {
                    let concrete_type = concrete_type.unwrap();
                    let bean_factory = Self::abstract_bean_factory_tokens(&concrete_type, abstract_type, profile_ident);
                    info!("Creating abstract bean container.");
                    let bean_container = Self::abstract_bean_container(&concrete_type, abstract_type, profile_ident);
                    info!("Creating abstract factory bean.");
                    let factory_bean = Self::abstract_factory_bean(&concrete_type, profile_ident, abstract_type, create_bean_tokens);
                    let injectable_code = quote! {
                        #bean_factory
                        #bean_container
                        #factory_bean
                    };
                    info!("finished creating injectable code.");
                    return injectable_code.into();
                } else if ident_type.is_some() {
                    let concrete_type = ident_type.unwrap();
                    let bean_factory = Self::abstract_bean_factory_tokens(&concrete_type, abstract_type, profile_ident);
                    info!("Creating abstract bean container.");
                    let bean_container = Self::abstract_bean_container(&concrete_type, abstract_type, profile_ident);
                    info!("Creating abstract factory bean.");
                    let factory_bean = Self::abstract_factory_bean(&concrete_type, profile_ident, abstract_type, create_bean_tokens);
                    let injectable_code = quote! {
                        #bean_factory
                        #bean_container
                        #factory_bean
                    };
                    info!("Finished creating injectable code.");
                    return injectable_code.into();
                }

                TokenStream::default().into()
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
            .filter(|bf| bf.ident_type.as_ref().is_some() || bf.concrete_type.as_ref().is_some())
            .for_each(|b| {
                ts.append_all(Self::create_concrete_bean_factories_for_bean(b));
            });

        self.get_abstract_factories().iter()
            .for_each(|b| {
                if !b.abstract_type.is_none() {
                    if b.ident_type.is_some() || b.concrete_type.is_some() {
                        ts.append_all(Self::create_abstract_bean_factories_for_bean(b));
                    }
                }
            });

        ts
    }

    fn create_bean_tokens<ConcreteTypeT: ToTokens>(
        bean_factory_info: &BeanFactoryInfo,
        profile_ident: &Ident,
        concrete_type: &ConcreteTypeT
    ) -> Option<TokenStream> {
        if bean_factory_info.factory_fn.is_some() {
            FactoryFnBeanFactoryGenerator::create_bean_tokens_default(bean_factory_info, profile_ident, concrete_type)
        } else {
            Self::create_bean_tokens_default(bean_factory_info, profile_ident, concrete_type)
        }
    }


    fn create_bean_tokens_default<ConcreteTypeT: ToTokens>(bean_factory_info: &BeanFactoryInfo,
                                                           profile_ident: &Ident, concrete_type: &ConcreteTypeT) -> Option<TokenStream> {
        if bean_factory_info.factory_fn.is_some() {
            log_message!("Skipping creation of bean factory for {} because has factory fn.",
                SynHelper::get_str(&concrete_type));
            return None;
        }

        let (field_idents, field_types, concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_idents, abstract_mutable_field_types, concrete_mutable_abstract)
            = bean_factory_info.get_field_singleton_types();

        let (prototype_field_idents, prototype_field_types, prototype_concrete_field,
            prototype_mutable_identifiers, prototype_mutable_field_types, prototype_concrete_mutable_type,
            prototype_abstract_field_idents, prototype_abstract_field_types, prototype_concrete_abstract_types,
            prototype_abstract_mutable_idents, prototype_abstract_mutable_field_types, prototype_concrete_mutable_abstract)
            = bean_factory_info.get_field_prototype_types();

        log_message!("Creating factory for profile {} for: {}.",
            SynHelper::get_str(profile_ident),
            SynHelper::get_str(&concrete_type)
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

                let inner = #concrete_type::new(
                    #(#prototype_field_idents,)* #(#prototype_mutable_identifiers,)*
                    #(#prototype_abstract_field_idents,)* #(#prototype_abstract_mutable_idents,)*
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
