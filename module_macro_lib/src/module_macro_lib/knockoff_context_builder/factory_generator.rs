use crate::module_macro_lib::module_tree::{AutowiredField, AutowireType, Bean, BeanDefinition, BeanDefinitionType, BeanType, Profile};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::__private::TokenStream2;
use syn::{Path, Type};
use codegen_utils::syn_helper::SynHelper;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::AspectParser;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::BeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;


pub trait FactoryGenerator: {

    fn generate_factory_tokens(&self) -> TokenStream;

    fn new_factory_generator(profile: Profile, bean_definitions: Vec<BeanDefinitionType>) -> Box<dyn TokenStreamGenerator> where Self: Sized;

    fn impl_listable_factory() -> TokenStream where Self: Sized {

        let new_listable_bean_factory = quote! {

            impl ListableBeanFactory {

                /// Important to note that if this was dyn Any + Send + Sync the type id would be different.
                /// Therefore, it is important to have it only be called with the impl type, or the dyn
                /// type for the abstract.
                fn add_bean_definition<T: 'static + Send + Sync>(&mut self, bean_defin: BeanDefinition<T>) {
                    self.singleton_bean_definitions.insert(
                        bean_defin.get_bean_type_id().clone(),
                        bean_defin.to_any()
                    );
                }

                /// Important to note that if this was dyn Any + Send + Sync the type id would be different.
                /// Therefore, it is important to have it only be called with the impl type, or the dyn
                /// type for the abstract.
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

    fn add_to(identifiers: &mut Vec<Ident>, types: &mut Vec<Type>, bean: &Bean) where Self: Sized {
        if bean.ident.is_some() {
            log_message!("Implementing listable bean factory. Including: {}.", bean.ident.to_token_stream().to_string().clone());
            identifiers.push(bean.ident.clone().unwrap());
        } else if bean.struct_type.is_some() {
            types.push(bean.struct_type.clone().unwrap());
        }
    }


}

pub struct FactoryGen {
    profile: Profile,
    beans: Vec<ProviderBean>
}

pub struct ProviderBean {
    bean: Bean,
    dep_type: Option<AutowireType>
}

impl TokenStreamGenerator for FactoryGen {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factory_tokens()
    }
}

impl FactoryGenerator for FactoryGen {

    fn generate_factory_tokens(&self) -> TokenStream {
        Self::new_listable_bean_factory(&self.beans, &self.profile)
    }

    fn new_factory_generator(profile: Profile, bean_definitions: Vec<BeanDefinitionType>) -> Box<dyn TokenStreamGenerator> where Self: Sized {
        let beans = bean_definitions.iter().flat_map(|b| {
            match b {
                BeanDefinitionType::Concrete { bean } => {
                    log_message!("{} is the number of trait types and {} is the number of deps for bean with id {}.",
                             bean.traits_impl.len(), bean.deps_map.len(), bean.id.as_str());
                    vec![ProviderBean {bean: bean.clone(), dep_type: None}]
                }
                BeanDefinitionType::Abstract { bean, dep_type } => {
                    vec![ProviderBean{ bean: bean.clone(), dep_type: Some(dep_type.clone()) }]
                }
            }
        }).collect::<Vec<ProviderBean>>();

        Box::new(
            Self {
                profile, beans
            }
        )
    }
}

impl FactoryGen {

    pub fn new_listable_bean_factory(beans_to_provide: &Vec<ProviderBean>, profile: &Profile) -> TokenStream {
        let profile_name_str = profile.profile.clone();

        let profile_name = Ident::new(profile_name_str.as_str(), Span::call_site());

        let (singleton_idents, singleton_types,
            mutable_types, mutable_idents,
            abstract_mutable_paths, abstract_paths)
            = Self::get_fill_ident_paths(beans_to_provide);

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
                        let next_bean_definition = <dyn BeanFactory<#singleton_idents, #profile_name>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn BeanFactory<#singleton_types, #profile_name>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn MutableBeanFactory<Mutex<#mutable_idents>, #profile_name>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn MutableBeanFactory<Mutex<#mutable_types>, #profile_name>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
                        listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn MutableBeanFactory<Mutex<Box<dyn #abstract_mutable_paths>>, #profile_name>>::get_bean(&listable_bean_factory);
                        listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn BeanFactory<Box<dyn #abstract_paths>, #profile_name>>::get_bean(&listable_bean_factory);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
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

                fn get_dyn_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<Box<T>>> {
                    let type_id = TypeId::of::<T>();
                    if self.contains_bean_type(&type_id) {
                        println!("Contains bean type!");
                        let downcast_result = self.singleton_bean_definitions[&type_id]
                            .inner.clone().downcast::<Box<T>>();
                        if downcast_result.is_ok() {
                            return Some(downcast_result.unwrap().clone());
                        }
                        return None;
                    }
                    println!("Does not contain bean type..");
                    None
                }

                fn get_mutable_dyn_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<Mutex<Box<T>>>> {
                    let type_id = TypeId::of::<T>();
                    if self.contains_mutable_bean_type(&type_id) {
                        println!("Contains bean type!");
                        let downcast_result = self.mutable_bean_definitions[&type_id]
                            .inner.clone().downcast::<Mutex<Box<T>>>();
                        if downcast_result.is_ok() {
                            return Some(downcast_result.unwrap().clone());
                        }
                        return None;
                    }
                    println!("Does not contain bean type..");
                    None
                }

                fn get_beans(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
                    let mut beans_vec = vec![];
                    self.singleton_bean_definitions.values().map(|s| s.inner.clone())
                        .for_each(|bean_def| beans_vec.push(bean_def));
                    beans_vec
                }

                fn get_mutable_beans(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
                    let mut beans_vec = vec![];
                    self.mutable_bean_definitions.values().map(|s| s.inner.clone())
                        .for_each(|bean_def| beans_vec.push(bean_def));
                    beans_vec
                }

            }

        };
        new_listable_bean_factory.into()
    }

    fn get_fill_ident_paths(beans_to_provide: &Vec<ProviderBean>) -> (Vec<Ident>, Vec<Type>, Vec<Type>, Vec<Ident>, Vec<Path>, Vec<Path>) {
        let mut singleton_idents = vec![];
        let mut singleton_types = vec![];
        let mut mutable_types = vec![];
        let mut mutable_idents = vec![];
        let mut abstract_mutable_paths: Vec<Path> = vec![];
        let mut abstract_paths: Vec<Path> = vec![];

        for provider_bean in beans_to_provide.iter() {
            let bean = &provider_bean.bean;
            provider_bean.dep_type.as_ref()
                .map(|autowire_type| {
                    bean.bean_type.as_ref().and_then(|bean_type| {
                        log_message!("Found bean type {:?}.", bean_type);
                        match bean_type {
                            BeanType::Singleton => {
                                log_message!("adding bean dep impl with type {} as singleton!", bean.id.clone());
                                if bean.mutable {
                                    autowire_type.item_impl.trait_.as_ref()
                                        .map(|t| abstract_mutable_paths.push(t.1.clone()));
                                } else {
                                    autowire_type.item_impl.trait_.as_ref()
                                        .map(|t| abstract_paths.push(t.1.clone()));
                                }
                            }
                            BeanType::Prototype => {
                                log_message!("Ignoring prototype bean {} when building bean factory.", bean.id.as_str());
                            }
                        };
                        None::<BeanType>
                    });
                    None::<BeanType>
                })
                .or_else(|| {
                    bean.bean_type.as_ref().and_then(|bean_type| {
                        log_message!("Found bean type {:?}.", bean_type);
                        match bean_type {
                            BeanType::Singleton => {
                                log_message!("adding bean dep impl with type {} as singleton!", bean.id.clone());
                                Self::add_to_ident_type(&mut singleton_idents, &mut singleton_types, &mut mutable_types, &mut mutable_idents, &bean);
                            }
                            BeanType::Prototype => {
                                log_message!("Ignoring prototype bean {} when building bean factory.", bean.id.as_str());
                            }
                        };
                        None::<BeanType>
                    });
                    None
                });
        }
        (singleton_idents, singleton_types, mutable_types, mutable_idents, abstract_mutable_paths, abstract_paths)
    }

    fn add_to_ident_type(mut singleton_idents: &mut Vec<Ident>,
                         mut singleton_types: &mut Vec<Type>,
                         mut mutable_types: &mut Vec<Type>,
                         mut mutable_idents: &mut Vec<Ident>,
                         bean: &Bean) {
        if bean.mutable {
            Self::add_to(&mut mutable_idents, &mut mutable_types, &bean);
        } else {
            Self::add_to(&mut singleton_idents, &mut singleton_types, &bean);
        }
    }
}