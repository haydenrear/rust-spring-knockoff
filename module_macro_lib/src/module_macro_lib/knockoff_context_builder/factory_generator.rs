use crate::module_macro_lib::module_tree::{AutowiredField, AutowireType, Bean, BeanDefinition, BeanDefinitionType, BeanType, Profile};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::__private::TokenStream2;
use syn::Type;
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

    fn add_to(identifiers: &mut Vec<Ident>, types: &mut Vec<Type>, bean: &Bean) where Self: Sized {
        if bean.ident.is_some() {
            log_message!("Implementing listable bean factory. Including: {}.", bean.ident.to_token_stream().to_string().clone());
            identifiers.push(bean.ident.clone().unwrap());
        } else if bean.struct_type.is_some() {
            types.push(bean.struct_type.clone().unwrap());
        }
    }
}

pub struct AbstractFactoryGenerator {
    profile: Profile,
    beans: Vec<(Bean, AutowireType)>
}

pub struct ConcreteFactoryGenerator{
    profile: Profile,
    beans: Vec<Bean>
}

impl TokenStreamGenerator for ConcreteFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factory_tokens()
    }
}

impl FactoryGenerator for ConcreteFactoryGenerator {

    fn generate_factory_tokens(&self) -> TokenStream {
        Self::new_listable_bean_factory(&self.beans, &self.profile)
    }

    fn new_factory_generator(profile: Profile, bean_definitions: Vec<BeanDefinitionType>) -> Box<dyn TokenStreamGenerator> where Self: Sized {
        let beans = bean_definitions.iter().flat_map(|b| {
            match b {
                BeanDefinitionType::Concrete { bean } => {
                    log_message!("{} is the number of trait types and {} is the number of deps for bean with id {}.",
                             bean.traits_impl.len(), bean.deps_map.len(), bean.id.as_str());
                    vec![bean.clone()]
                }
                _ => {
                    vec![]
                }
            }
        }).collect::<Vec<Bean>>();

        Box::new(
            Self {
                profile, beans
            }
        )
    }
}

impl TokenStreamGenerator for AbstractFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factory_tokens()
    }
}

impl FactoryGenerator for AbstractFactoryGenerator {

    fn generate_factory_tokens(&self) -> TokenStream {
        Self::new_listable_bean_factory(self.beans.clone(), self.profile.clone())
    }

    fn new_factory_generator(profile: Profile, bean_definitions: Vec<BeanDefinitionType>) -> Box<dyn TokenStreamGenerator> where Self: Sized {
        let beans = bean_definitions.iter().flat_map(|b| {
            match b {
                BeanDefinitionType::Abstract { bean, dep_type } => {
                    log_message!("{} is the bean id", bean.id.clone());
                    log_message!("{} is trait impl", dep_type.item_impl.to_token_stream().to_string().as_str());
                    dep_type.item_impl.trait_.clone().map(|trait_found| {
                        log_message!("{} is the abstract bean type.", SynHelper::get_str(trait_found.clone().1));
                    });
                    log_message!("{} is the number of autowire types for abstract.", bean.traits_impl.len());
                    log_message!("{} is the number of deps for abstract.", bean.deps_map.len());
                    vec![(bean.clone(), dep_type.clone())]
                }
                _ => {
                    vec![]
                }
            }
        }).collect::<Vec<(Bean, AutowireType)>>();

        Box::new(
            Self {
                profile, beans
            }
        )
    }
}

impl ConcreteFactoryGenerator {

    pub fn new_listable_bean_factory(beans_to_provide: &Vec<Bean>, profile: &Profile) -> TokenStream {
        let profile_name_str = profile.profile.clone();

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
                    BeanType::Singleton => {
                        log_message!("adding bean dep impl with type {} as singleton!", bean.id.clone());
                        if bean.mutable {
                            Self::add_to(&mut mutable_idents, &mut mutable_types, &bean);
                        } else {
                            Self::add_to(&mut singleton_idents, &mut singleton_types, &bean);
                        }
                    }
                    BeanType::Prototype => {
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
}

impl AbstractFactoryGenerator {

    pub fn new_listable_bean_factory(beans_to_provide: Vec<(Bean, AutowireType)>, profile: Profile) -> TokenStream {
        let profile_name_str = profile.profile;

        let profile_name = Ident::new(profile_name_str.as_str(), Span::call_site());

        let mut singleton_idents = vec![];
        let mut singleton_types = vec![];
        let mut prototype_idents = vec![];
        let mut prototype_types = vec![];
        let mut mutable_types = vec![];
        let mut mutable_idents = vec![];

        for bean_val in beans_to_provide.iter() {

            let bean = &bean_val.0;

            bean.bean_type.as_ref().and_then(|bean_type| {
                log_message!("Found bean type {:?}.", bean_type);
                match bean_type {
                    BeanType::Singleton => {
                        log_message!("adding bean dep impl with type {} as singleton!", bean.id.clone());
                        if bean.mutable {
                            Self::add_to(&mut mutable_idents, &mut mutable_types, bean);
                        } else {
                            Self::add_to(&mut singleton_idents, &mut singleton_types, bean);
                        }
                    }
                    BeanType::Prototype => {
                        log_message!("adding bean dep impl with type {} as prototype!", bean.id.clone());
                        if bean.mutable {
                            Self::add_to(&mut mutable_idents, &mut mutable_types, bean);
                        } else {
                            Self::add_to(&mut prototype_idents, &mut prototype_types, bean);
                        }
                    }
                };
                None::<BeanType>
            });
        }

        // let new_listable_bean_factory = quote! {
        //
        //     pub struct #profile_name {
        //     }
        //
        //     impl Profile for #profile_name {
        //         fn name() -> String {
        //             String::from(#profile_name_str)
        //         }
        //     }
        //
        //     impl AbstractListableFactory<#profile_name> for ListableBeanFactory {
        //
        //         fn new() -> Self {
        //             let mut singleton_bean_definitions: HashMap<TypeId, BeanDefinition<dyn Any + Send + Sync>> = HashMap::new();
        //             let mut mutable_bean_definitions: HashMap<TypeId, MutableBeanDefinition<dyn Any + Send + Sync>> = HashMap::new();
        //             let mut listable_bean_factory = ListableBeanFactory {
        //                 singleton_bean_definitions,
        //                 mutable_bean_definitions
        //             };
        //             #(
        //                 let next_bean_definition = <BeanDefinition<#singleton_idents >>::get_bean(&listable_bean_factory);
        //                 println!("Adding next bean definition {:?}.", next_bean_definition);
        //                 listable_bean_factory.add_bean_definition(next_bean_definition);
        //             )*
        //             #(
        //                 let next_bean_definition = <BeanDefinition<#singleton_types>>::get_bean(&listable_bean_factory);
        //                 println!("Adding next bean definition {:?}.", next_bean_definition);
        //                 listable_bean_factory.add_bean_definition(next_bean_definition);
        //             )*
        //             #(
        //                 let next_bean_definition = <MutableBeanDefinition<Mutex<#mutable_idents>>>::get_bean(&listable_bean_factory);
        //                 println!("Adding next bean definition {:?}.", next_bean_definition);
        //                 listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
        //             )*
        //             #(
        //                 let next_bean_definition = <MutableBeanDefinition<Mutex<#mutable_types>>>::get_bean(&listable_bean_factory);
        //                 println!("Adding next bean definition {:?}.", next_bean_definition);
        //                 listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
        //             )*
        //             listable_bean_factory
        //         }
        //
        //         fn get_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        //             let type_id = TypeId::of::<T>();
        //             if self.contains_bean_type(&type_id) {
        //                 println!("Contains bean type!");
        //                 let downcast_result = self.singleton_bean_definitions[&type_id]
        //                     .inner.clone().downcast::<T>();
        //                 if downcast_result.is_ok() {
        //                     return Some(downcast_result.unwrap().clone());
        //                 }
        //                 return None;
        //             }
        //             println!("Does not contain bean type..");
        //             None
        //         }
        //
        //         fn get_mutable_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<Mutex<T>>> {
        //             let type_id = TypeId::of::<T>();
        //             if self.contains_mutable_bean_type(&type_id) {
        //                 println!("Contains bean type!");
        //                 let downcast_result = self.mutable_bean_definitions[&type_id]
        //                     .inner.clone().downcast::<Mutex<T>>();
        //                 if downcast_result.is_ok() {
        //                     return Some(downcast_result.unwrap().clone());
        //                 }
        //                 return None;
        //             }
        //             println!("Does not contain bean type..");
        //             None
        //         }
        //
        //         fn get_beans(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        //             let mut beans_vec = vec![];
        //             self.singleton_bean_definitions.values().map(|s| s.inner.clone())
        //                 .for_each(|bean_def| beans_vec.push(bean_def));
        //             beans_vec
        //         }
        //
        //         fn get_mutable_beans(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        //             let mut beans_vec = vec![];
        //             self.mutable_bean_definitions.values().map(|s| s.inner.clone())
        //                 .for_each(|bean_def| beans_vec.push(bean_def));
        //             beans_vec
        //         }
        //
        //     }
        //
        // };
        // new_listable_bean_factory.into()
        TokenStream::default()
    }
}