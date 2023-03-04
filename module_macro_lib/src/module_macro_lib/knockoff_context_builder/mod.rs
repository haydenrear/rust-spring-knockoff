use std::any::{Any, TypeId};
use std::collections::HashMap;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::__private::TokenStream2;
use syn::Type;
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::module_tree::{BeanType, Bean, Profile};

use knockoff_logging::{initialize_log, use_logging};
use crate::module_macro_lib::aspect::AspectParser;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub struct ApplicationContextGenerator;

impl ApplicationContextGenerator {

    pub fn create_application_context() -> TokenStream {
        let ts = quote! {
            use module_macro_lib::module_macro_lib::knockoff_context::{AbstractListableFactory, ApplicationContext, Profile, ContainsBeans};
            use std::sync::Mutex;
        };
        ts.into()
    }

    pub fn create_bean_factory() -> TokenStream {
        let ts = quote! {
            pub fn get_type_id_from_gen<T: ?Sized + 'static>() -> TypeId {
                TypeId::of::<T>()
            }

            #[derive(Debug)]
            pub struct BeanDefinition<T: ?Sized> {
                pub inner: Arc<T>
            }

            #[derive(Debug)]
            pub struct MutableBeanDefinition<T: ?Sized> {
                pub inner: Arc<T>
            }

            #[derive(Debug)]
            pub struct PrototypeBeanDefinition<T: ?Sized> {
                pub inner: Arc<T>
            }

            impl <T: 'static + Send + Sync> MutableBeanDefinition<T> {
                fn to_any(&self) -> MutableBeanDefinition<dyn Any + Send + Sync> {
                    let inner: Arc<dyn Any + Send + Sync> = self.inner.clone() as Arc<dyn Any + Send + Sync>;
                    MutableBeanDefinition {
                        inner
                    }
                }

                fn get_bean_type_id(&self) -> TypeId {
                    self.inner.type_id().clone()
                }
            }

            impl <T: 'static + Send + Sync> PrototypeBeanDefinition<T> {
                fn to_any(&self) -> PrototypeBeanDefinition<dyn Any + Send + Sync> {
                    let inner: Arc<dyn Any + Send + Sync> = self.inner.clone() as Arc<dyn Any + Send + Sync>;
                    PrototypeBeanDefinition {
                        inner
                    }
                }

                fn get_bean_type_id(&self) -> TypeId {
                    self.inner.deref().type_id().clone()
                }
            }

            impl <T: 'static + Send + Sync> BeanDefinition<T> {
                fn to_any(&self) -> BeanDefinition<dyn Any + Send + Sync> {
                    let inner: Arc<dyn Any + Send + Sync> = self.inner.clone() as Arc<dyn Any + Send + Sync>;
                    BeanDefinition {
                        inner
                    }
                }

                fn get_bean_type_id(&self) -> TypeId {
                    self.inner.deref().type_id().clone()
                }
            }

            pub trait BeanFactory<T: 'static + Send + Sync + ?Sized> {
                fn get_bean(&self) -> BeanDefinition<T>;
            }

            pub trait MutableBeanFactory<T: 'static + Send + Sync + ?Sized> {
                fn get_bean(&self) -> MutableBeanDefinition<T>;
            }

            pub trait MutableFactoryBean<T: 'static + Send + Sync + ?Sized> {
                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> MutableBeanDefinition<T>;
                fn get_bean_type_id(&self) -> TypeId;
                fn is_singleton() -> bool;
            }

            pub trait FactoryBean<T: 'static + Send + Sync + ?Sized> {
                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<T>;
                fn get_bean_type_id(&self) -> TypeId;
                fn is_singleton() -> bool;
            }

            #[derive(Default)]
            pub struct ListableBeanFactory {
                singleton_bean_definitions: HashMap<TypeId, BeanDefinition<dyn Any + Send + Sync>>,
                mutable_bean_definitions: HashMap<TypeId, MutableBeanDefinition<dyn Any + Send + Sync>>
            }

            impl ContainsBeans for ListableBeanFactory {

                fn contains_bean_type(&self, type_id: &TypeId) -> bool {
                    for bean_def in self.singleton_bean_definitions.iter() {
                        println!("Checking if {:?}", bean_def.1);
                        if bean_def.0 == type_id {
                            return true;
                        }
                    }
                    false
                }

                fn contains_mutable_bean_type(&self, type_id: &TypeId) -> bool {
                    for bean_def in self.mutable_bean_definitions.iter() {
                        println!("Checking if {:?}", bean_def.1);
                        if bean_def.0 == type_id {
                            return true;
                        }
                    }
                    false
                }

                fn get_bean_types(&self) -> Vec<TypeId> {
                    self.singleton_bean_definitions.keys()
                        .map(|type_id| type_id.clone())
                        .collect::<Vec<TypeId>>()
                }

                fn get_mutable_bean_types(&self) -> Vec<TypeId> {
                    self.mutable_bean_definitions.keys()
                        .map(|type_id| type_id.clone())
                        .collect::<Vec<TypeId>>()
                }

                fn contains_type<T: 'static + Send + Sync>(&self) -> bool {
                    let type_id_to_search = TypeId::of::<T>();
                    self.singleton_bean_definitions.keys()
                        .any(|t| t.clone() == type_id_to_search.clone())
                }

                fn contains_mutable_type<T: 'static + Send + Sync>(&self) -> bool {
                    let type_id_to_search = TypeId::of::<T>();
                    self.mutable_bean_definitions.keys()
                        .any(|t| t.clone() == type_id_to_search.clone())
                }
            }

        };
        ts.into()
    }

    pub fn new_listable_bean_factory(beans_to_provide: Vec<Bean>, profile: Profile) -> TokenStream {
        let profile_name_str = profile.profile;

        let profile_name = Ident::new(profile_name_str.as_str(), Span::call_site());

        let mut singleton_idents = vec![];
        let mut singleton_types = vec![];
        let mut prototype_idents = vec![];
        let mut prototype_types = vec![];
        let mut mutable_types = vec![];
        let mut mutable_idents = vec![];

        for bean in beans_to_provide.iter() {

            bean.bean_type.as_ref().and_then(|bean_type| {
                log_message!("Found bean type {:?}.", bean_type);
                match bean_type {
                    BeanType::Singleton(b, _) => {
                        log_message!("adding bean dep impl with type {} as singleton!", bean.id.clone());
                        if bean.mutable {
                            Self::add_to(&mut mutable_idents, &mut mutable_types, &bean);
                        } else {
                            Self::add_to(&mut singleton_idents, &mut singleton_types, &bean);
                        }
                    }
                    BeanType::Prototype(_, _) => {
                        log_message!("adding bean dep impl with type {} as prototype!", bean.id.clone());
                        if bean.mutable {
                            Self::add_to(&mut mutable_idents, &mut mutable_types, &bean);
                        } else {
                            Self::add_to(&mut prototype_idents, &mut prototype_types, &bean);
                        }
                    }
                };
                None::<BeanType>
            });
        }

        let new_listable_bean_factory = quote! {

            pub struct #profile_name {
            }

            impl Profile for #profile_name {
                fn name() -> String {
                    String::from(#profile_name_str)
                }
            }

            impl AbstractListableFactory<#profile_name> for ListableBeanFactory {

                fn new() -> Self {
                    let mut singleton_bean_definitions: HashMap<TypeId, BeanDefinition<dyn Any + Send + Sync>> = HashMap::new();
                    let mut mutable_bean_definitions: HashMap<TypeId, MutableBeanDefinition<dyn Any + Send + Sync>> = HashMap::new();
                    let mut listable_bean_factory = ListableBeanFactory {
                        singleton_bean_definitions,
                        mutable_bean_definitions
                    };
                    #(
                        let next_bean_definition = <BeanDefinition<#singleton_idents >>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <BeanDefinition<#singleton_types>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <BeanDefinition<#prototype_idents>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <BeanDefinition<#prototype_types>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <MutableBeanDefinition<Mutex<#mutable_idents>>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <MutableBeanDefinition<Mutex<#mutable_types>>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
                    )*
                    listable_bean_factory
                }

                fn get_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
                    let type_id = TypeId::of::<T>();
                    if self.contains_bean_type(&type_id) {
                        println!("Contains bean type!");
                        let downcast_result = self.singleton_bean_definitions[&type_id]
                            .inner.clone().downcast::<T>();
                        if downcast_result.is_ok() {
                            return Some(downcast_result.unwrap().clone());
                        }
                        return None;
                    }
                    println!("Does not contain bean type..");
                    None
                }

                fn get_mutable_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<Mutex<T>>> {
                    let type_id = TypeId::of::<T>();
                    if self.contains_mutable_bean_type(&type_id) {
                        println!("Contains bean type!");
                        let downcast_result = self.mutable_bean_definitions[&type_id]
                            .inner.clone().downcast::<Mutex<T>>();
                        if downcast_result.is_ok() {
                            return Some(downcast_result.unwrap().clone());
                        }
                        return None;
                    }
                    println!("Does not contain bean type..");
                    None
                }

                fn get_beans<T: 'static + Send + Sync>(&self) -> Vec<Arc<T>> {
                    vec![]
                }
            }

            impl ListableBeanFactory {

                fn add_bean_definition<T: 'static + Send + Sync>(&mut self, bean_defin: BeanDefinition<T>) {
                    self.singleton_bean_definitions.insert(
                        bean_defin.get_bean_type_id().clone(),
                        bean_defin.to_any()
                    );
                }

                fn add_mutable_bean_definition<T: 'static + Send + Sync>(&mut self, bean_defin: MutableBeanDefinition<T>) {
                    self.mutable_bean_definitions.insert(
                        bean_defin.get_bean_type_id().clone(),
                        bean_defin.to_any()
                    );
                }


            }
        };
        new_listable_bean_factory.into()
    }

    fn add_to(singleton_idents: &mut Vec<Ident>, singleton_types: &mut Vec<Type>, bean: &&Bean) {
        if bean.ident.is_some() {
            log_message!("Implementing listable bean factory. Including: {}.", bean.ident.to_token_stream().to_string().clone());
            singleton_idents.push(bean.ident.clone().unwrap());
        } else if bean.struct_type.is_some() {
            singleton_types.push(bean.struct_type.clone().unwrap());
        }
    }

    pub fn gen_autowire_code_gen_concrete<T: ToTokens>(
        field_types: &Vec<Type>,
        field_idents: &Vec<Ident>,
        struct_type: &T,
        mutable_identifiers: &Vec<Ident>,
        mutable_field_types: &Vec<Type>
    ) -> TokenStream2 {
        log_message!("Creating bean factory with the following mutable field types: ");
        mutable_field_types.iter().for_each(|m| {
            log_message!("{} is the mutable field type.", SynHelper::get_str(m.clone()));
        });
        log_message!("Creating bean factory with the following field types: ");
        field_types.iter().for_each(|m| {
            log_message!("{} is the mutable field type.", SynHelper::get_str(m.clone()));
        });
        let injectable_code = quote! {

                impl BeanFactory<#struct_type> for ListableBeanFactory {
                    fn get_bean(&self) -> BeanDefinition<#struct_type> {
                        let this_component = <BeanDefinition<#struct_type>>::get_bean(&self);
                        this_component
                    }
                }

                impl FactoryBean<#struct_type> for BeanDefinition<#struct_type> {

                    fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<#struct_type> {
                        let mut inner = #struct_type::default();
                        #(
                            let bean_def: BeanDefinition<#field_types> = <ListableBeanFactory as BeanFactory<#field_types>>::get_bean(listable_bean_factory);
                            let arc_bean_def: Arc<#field_types> = bean_def.inner;
                            inner.#field_idents = arc_bean_def.clone();
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>> = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>>>::get_bean(listable_bean_factory);
                            let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                            inner.#mutable_identifiers = arc_bean_def.clone();
                        )*
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


    pub fn gen_autowire_mutable_code_gen_concrete<T: ToTokens>(
        field_types: &Vec<Type>,
        field_idents: &Vec<Ident>,
        struct_type: &T,
        mutable_identifiers: &Vec<Ident>,
        mutable_field_types: &Vec<Type>
    ) -> TokenStream2 {
        log_message!("Creating mutable bean factory with the following mutable field types: ");
        mutable_field_types.iter().for_each(|m| {
            log_message!("{} is the mutable field type.", SynHelper::get_str(m.clone()));
        });
        log_message!("Creating mutable bean factory with the following field types: ");
        field_types.iter().for_each(|m| {
            log_message!("{} is the mutable field type.", SynHelper::get_str(m.clone()));
        });
        let injectable_code = quote! {

                impl MutableBeanFactory<Mutex<#struct_type>> for ListableBeanFactory {
                    fn get_bean(&self) -> MutableBeanDefinition<Mutex<#struct_type>> {
                        let this_component = <MutableBeanDefinition<Mutex<#struct_type>>>::get_bean(&self);
                        this_component
                    }
                }

                impl MutableFactoryBean<Mutex<#struct_type>> for MutableBeanDefinition<Mutex<#struct_type>> {

                    fn get_bean(listable_bean_factory: &ListableBeanFactory) -> MutableBeanDefinition<Mutex<#struct_type>> {
                        let mut inner = #struct_type::default();
                        #(
                            let bean_def: BeanDefinition<#field_types> = <ListableBeanFactory as BeanFactory<#field_types>>::get_bean(listable_bean_factory);
                            let arc_bean_def: Arc<#field_types> = bean_def.inner;
                            inner.#field_idents = arc_bean_def.clone();
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>> = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>>>::get_bean(listable_bean_factory);
                            let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                            inner.#mutable_identifiers = arc_bean_def.clone();
                        )*
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

    pub fn gen_autowire_code_gen_abstract<T: ToTokens>(field_types: &Vec<Type>, field_idents: &Vec<Ident>, struct_type: &T)
                                                       -> TokenStream2 {
        let injectable_code = quote! {

                // impl BeanFactory<#struct_type> for ListableBeanFactory {
                //     fn get_bean(&self) -> BeanDefinition<#struct_type> {
                //         let this_component = <BeanDefinition<#struct_type>>::get_bean(&self);
                //         this_component
                //     }
                // }
                //
                // impl FactoryBean<#struct_type> for BeanDefinition<#struct_type> {
                //
                //     fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<#struct_type> {
                //         let mut inner = #struct_type::default_impls();
                //         #(
                //             inner.#field_idents = ListableBeanFactory::<#field_types>::get_bean(listable_bean_factory);
                //         )*
                //         Self {
                //             inner: Arc::new(inner)
                //         }
                //     }
                //
                //     fn get_bean_type_id(&self) -> TypeId {
                //         self.inner.deref().type_id().clone()
                //     }
                //
                //     fn is_singleton() -> bool {
                //         true
                //     }
                //
                // }

        };

        injectable_code.into()
    }

    pub fn finish_abstract_factory(profiles_names: Vec<String>) -> TokenStream2 {

        let profiles = profiles_names.iter()
            .map(|p| Ident::new(p.as_str(), Span::call_site()))
            .collect::<Vec<Ident>>();

        let injectable_code = quote! {

            pub struct AppCtx {
                factories: HashMap<String,ListableBeanFactory>,
                profiles: Vec<String>
            }

            impl ApplicationContext for AppCtx {

                fn get_bean_by_type_id<T,P>(&self, type_id: TypeId) -> Option<Arc<T>>
                where P: Profile, T: 'static + Send + Sync
                {
                    self.factories.get(&P::name())
                        .unwrap()
                        .get_bean_definition::<T>()
                }

                fn get_bean_by_qualifier<T,P>(&self, qualifier: String) -> Option<Arc<T>>
                where P: Profile, T: 'static + Send + Sync
                {
                    None
                }

                fn get_bean<T,P>(&self) -> Option<Arc<T>>
                where P: Profile, T: 'static + Send + Sync
                {
                    None
                }

                fn get_beans<T>(&self) -> Vec<Arc<T>>
                where T: 'static + Send + Sync
                {
                    vec![]
                }

                fn new() -> Self {
                    let mut factories = HashMap::new();
                    #(
                        let profile = AbstractListableFactory::<#profiles>::new();
                        factories.insert(String::from(#profiles_names), profile);
                    )*
                    let mut profiles = vec![];
                    #(
                        profiles.push(String::from(#profiles_names));
                    )*
                    Self {
                        factories,
                        profiles
                    }
                }

            }
        };

        injectable_code.into()
    }

}



