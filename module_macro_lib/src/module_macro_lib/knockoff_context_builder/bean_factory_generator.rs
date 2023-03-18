use std::rc::Rc;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::token::Mut;
use syn::{Path, Type};
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use module_macro_shared::bean::{Bean, BeanPath};
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::{AbstractBeanFactoryInfo, BeanFactoryInfo, BeanFactoryInfoFactory, ConcreteBeanFactoryInfo};

use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::dependency::{AutowiredField, AutowireType, DepType};
use module_macro_shared::profile_tree::ProfileBuilder;
use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub trait BeanFactoryGenerator: TokenStreamGenerator {

    fn create_concrete_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream;

    fn create_abstract_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream;

    fn new_bean_factory_generators(concrete_beans: Vec<BeanFactoryInfo>, abstract_beans: Vec<BeanFactoryInfo>) -> Vec<Box<dyn TokenStreamGenerator>> {
        vec![
            Box::new(MutableBeanFactoryGenerator::new_bean_factory_generator(concrete_beans.clone(), abstract_beans.clone())) as Box<dyn TokenStreamGenerator>,
            Box::new(FactoryBeanBeanFactoryGenerator::new_bean_factory_generator(concrete_beans.clone(), abstract_beans.clone())) as Box<dyn TokenStreamGenerator>,
            Box::new(PrototypeBeanFactoryGenerator::new_bean_factory_generator(concrete_beans.clone(), abstract_beans.clone())) as Box<dyn TokenStreamGenerator>,
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
    ) -> TokenStream {

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

    fn create_concrete_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream{

        let profile_ident = &bean_factory_info.get_profile_ident();
        let concrete_type = bean_factory_info.get_concrete_type();

        let create_bean_tokens = Self::create_bean_tokens(bean_factory_info, profile_ident, &concrete_type);

        let injectable_code = quote! {

                impl MutableBeanFactory<Mutex<#concrete_type>, #profile_ident> for ListableBeanFactory {
                    type U = Mutex<#concrete_type>;
                    fn get_bean(&self) -> MutableBeanDefinition<Mutex<#concrete_type >> {
                        let this_component = <MutableBeanDefinition<Mutex<#concrete_type >>>::get_bean(&self);
                        this_component
                    }
                }

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

        };

        injectable_code.into()
    }

    fn create_abstract_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream {

        let profile_ident = &bean_factory_info.get_profile_ident();
        let concrete_type = bean_factory_info.get_concrete_type();

        log_message!("Building container");

        let create_bean_tokens = Self::create_bean_tokens(bean_factory_info, profile_ident, &concrete_type);


        let abstract_type = bean_factory_info.abstract_type.as_ref().unwrap();

        /// If you implement the dyn factory by returning the concrete type, then you save the bean
        /// as the concrete type. You can then implement the same way to get the type id easily.
        let injectable_code = quote! {

            impl MutableBeanFactory<Mutex<Box<dyn #abstract_type>>, #profile_ident> for ListableBeanFactory {
                type U = Mutex<Box<dyn #abstract_type>>;
                fn get_bean(&self) -> MutableBeanDefinition<Self::U> {
                    <MutableBeanDefinition<Mutex<Box<dyn #abstract_type>>>>::get_bean(&self)
                }
            }

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

        };

        injectable_code.into()
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

pub struct FactoryBeanBeanFactoryGenerator {
    concrete_bean_factories: Vec<BeanFactoryInfo>,
    abstract_bean_factories: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for FactoryBeanBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

impl BeanFactoryGenerator for FactoryBeanBeanFactoryGenerator {
    fn create_concrete_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream{

        let profile_ident = &bean_factory_info.get_profile_ident();
        let concrete_type = bean_factory_info.get_concrete_type();
        let create_bean_tokens = Self::create_bean_tokens(bean_factory_info, profile_ident, &concrete_type);

        let injectable_code = quote! {

                impl BeanFactory<#concrete_type, #profile_ident> for ListableBeanFactory {
                    type U = #concrete_type;
                    fn get_bean(&self) -> BeanDefinition<#concrete_type> {
                        let this_component = <BeanDefinition<#concrete_type >>::get_bean(&self);
                        this_component
                    }
                }

                impl BeanContainer<#concrete_type> for ListableBeanFactory {
                    type U = #concrete_type;
                    fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                        self.singleton_bean_definitions.get(&TypeId::of::<Arc<#concrete_type >>())
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

                impl BeanContainerProfile<#concrete_type, #profile_ident> for ListableBeanFactory {
                    type U = Mutex<#concrete_type>;
                    fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                        self.singleton_bean_definitions.get(&TypeId::of::<Arc<#concrete_type >>())
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

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

        };

        injectable_code.into()
    }

    fn create_abstract_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream {

        log_message!("Building factory generator for {}", SynHelper::get_str(&bean_factory_info.abstract_type.as_ref().unwrap()));

        let struct_type: Ident = bean_factory_info.concrete_type
            .as_ref()
            .map(|t| Ident::new(t.to_token_stream().to_string().as_str(), Span::call_site()))
            .or(bean_factory_info.ident_type.clone())
            .unwrap();

        let abstract_type: &Path = bean_factory_info.abstract_type.as_ref().unwrap();

        let profile_ident = &bean_factory_info.get_profile_ident();
        let concrete_type = bean_factory_info.get_concrete_type();

        let create_bean_tokens = Self::create_bean_tokens(bean_factory_info, profile_ident, &concrete_type);

        let injectable_code = quote! {

                impl BeanFactory<dyn #abstract_type, #profile_ident> for ListableBeanFactory {
                    type U = #struct_type;
                    fn get_bean(&self) -> BeanDefinition<#struct_type> {
                       let bean_def: BeanDefinition<#struct_type> = <BeanDefinition<dyn #abstract_type> as FactoryBean<dyn #abstract_type, #profile_ident>>::get_bean(&self);
                        bean_def
                    }
                }

                impl BeanContainer<dyn #abstract_type> for ListableBeanFactory {
                    type U = #struct_type;
                    fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                        let type_id = TypeId::of::<Arc<dyn #abstract_type>>();
                        self.singleton_bean_definitions.get(&type_id)
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

                impl BeanContainerProfile<dyn #abstract_type, #profile_ident> for ListableBeanFactory {
                    type U = #struct_type;
                    fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                        let type_id = TypeId::of::<Arc<dyn #abstract_type>>();
                        self.singleton_bean_definitions.get(&type_id)
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

                impl FactoryBean<dyn #abstract_type, #profile_ident> for BeanDefinition<dyn #abstract_type> {
                    type U = #struct_type;

                    fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<#struct_type> {

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

        };

        injectable_code.into()
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
    fn create_concrete_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream {

        let injectable_code = quote! {

                // impl PrototypeFactoryBean<#struct_type, #default_profile> for ListableBeanFactory {
                //
                //     fn get_prototype_bean(&self) -> PrototypeBeanDefinition<#struct_type> {
                //         let mut inner = #struct_type::default();
                //         #(
                //             let bean_def: BeanDefinition<#field_types> = <ListableBeanFactory as BeanFactory<#field_types, #default_profile>>::get_bean(&self);
                //             let arc_bean_def: Arc<#field_types> = bean_def.inner;
                //             inner.#field_idents = arc_bean_def.clone();
                //         )*
                //         #(
                //             let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>>
                //                 = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>, #default_profile>>::get_bean(&self);
                //             let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                //             inner.#mutable_identifiers = arc_bean_def.clone();
                //         )*
                //         PrototypeBeanDefinition {
                //             inner: Arc::new(inner)
                //         }
                //     }
                //
                //     fn get_bean_type_id() -> TypeId {
                //         TypeId::of::<#struct_type>().clone()
                //     }
                //
                // }

        };

        injectable_code.into()
    }
    fn create_abstract_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream {
        // log_message!("Creating bean factory with the following mutable field types: ");
        // mutable_field_types.iter().for_each(|m| {
        //     log_message!("{} is the mutable field type.", SynHelper::get_str(m.clone()));
        // });
        // log_message!("Creating bean factory with the following field types: ");
        // field_types.iter().for_each(|m| {
        //     log_message!("{} is the mutable field type.", SynHelper::get_str(m.clone()));
        // });
        let injectable_code = quote! {

                // impl PrototypeFactoryBean<dyn #abstract_type> for ListableBeanFactory {
                //
                //     fn get_prototype_bean(&self) -> PrototypeBeanDefinition<Box<dyn #abstract_type>> {
                //         let mut inner = #concrete_type::default();
                //         #(
                //             let bean_def: BeanDefinition<#field_types> = <ListableBeanFactory as BeanFactory<#field_types>>::get_bean(&self);
                //             let arc_bean_def: Arc<#field_types> = bean_def.inner;
                //             inner.#field_idents = arc_bean_def.clone();
                //         )*
                //         #(
                //             let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>> = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>>>::get_bean(&self);
                //             let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                //             inner.#mutable_field_idents = arc_bean_def.clone();
                //         )*
                //         PrototypeBeanDefinition {
                //             inner: Arc::new(inner)
                //         }
                //     }
                //
                //     fn get_bean_type_id() -> TypeId {
                //         TypeId::of::<#concrete_type>().clone()
                //     }
                //
                // }

        };

        injectable_code.into()
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
