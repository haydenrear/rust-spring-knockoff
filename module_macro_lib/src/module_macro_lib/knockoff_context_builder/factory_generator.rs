use std::any::{Any, TypeId};
use std::collections::HashMap;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::__private::{str, TokenStream2};
use syn::{parse2, Path, Type};
use codegen_utils::syn_helper::SynHelper;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::AspectParser;
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType, BeanType};
use module_macro_shared::dependency::{AutowiredField, DependencyDescriptor};
use module_macro_shared::profile_tree::ProfileBuilder;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::BeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

pub trait FactoryGenerator: {

    fn generate_factory_tokens(&self) -> TokenStream;

    fn new_factory_generator(profile: ProfileBuilder, bean_definitions: Vec<BeanDefinitionType>) -> Box<dyn TokenStreamGenerator> where Self: Sized;

    fn impl_listable_factory() -> TokenStream where Self: Sized {

        let new_listable_bean_factory = quote! {

            impl ListableBeanFactory {

                /// Important to note that if this was dyn Any + Send + Sync the type id would be different.
                /// Therefore, it is important to have it only be called with the impl type, or the dyn
                /// type for the abstract.
                fn add_bean_definition<T: 'static + Send + Sync>(&mut self, bean_defin: BeanDefinition<T>) {
                    self.singleton_bean_definitions.insert(
                        bean_defin.get_bean_type_id(),
                        bean_defin.to_any()
                    );
                }

                fn add_bean_definition_type_id<T: 'static + Send + Sync>(&mut self, bean_defin: BeanDefinition<T>, type_id: TypeId) {
                    println!("Adding bean definition type id.");
                    self.singleton_bean_definitions.insert(
                        type_id,
                        bean_defin.to_any()
                    );
                }

                fn add_mutable_bean_definition_type_id<T: 'static + Send + Sync>(&mut self, bean_defin: MutableBeanDefinition<T>, type_id: TypeId) {
                    self.mutable_bean_definitions.insert(
                        type_id,
                        bean_defin.to_any()
                    );
                }

                /// Important to note that if this was dyn Any + Send + Sync the type id would be different.
                /// Therefore, it is important to have it only be called with the impl type, or the dyn
                /// type for the abstract.
                fn add_mutable_bean_definition<T: 'static + Send + Sync>(&mut self, bean_defin: MutableBeanDefinition<T>) {
                    self.mutable_bean_definitions.insert(
                        bean_defin.get_bean_type_id(),
                        bean_defin.to_any()
                    );
                }
            }

        };

        new_listable_bean_factory.into()
    }

    fn add_to(identifiers: &mut Vec<Ident>, types: &mut Vec<Type>, bean: &BeanDefinition) where Self: Sized {
        if bean.ident.is_some() {
            log_message!("Implementing listable bean factory. Including: {}.", bean.ident.to_token_stream().to_string().clone());
            identifiers.push(bean.ident.clone().unwrap());
        } else if bean.struct_type.is_some() {
            types.push(bean.struct_type.clone().unwrap());
        }
    }
}

pub struct FactoryGen {
    profile: ProfileBuilder,
    beans: Vec<ProviderBean>
}

pub struct ProviderBean {
    bean: BeanDefinition,
    autowire_type: Option<DependencyDescriptor>
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

    fn new_factory_generator(profile: ProfileBuilder, bean_definitions: Vec<BeanDefinitionType>)
                             -> Box<dyn TokenStreamGenerator> where Self: Sized
    {
        let beans = bean_definitions.iter().flat_map(|b| {
            match b {
                BeanDefinitionType::Concrete { bean } => {
                    log_message!("{} is the number of trait types and {} is the number of deps for bean with id {}.",
                             bean.traits_impl.len(), bean.deps_map.len(), bean.id.as_str());
                    vec![ProviderBean {bean: bean.clone(), autowire_type: None}]
                }
                BeanDefinitionType::Abstract { bean, dep_type } => {
                    vec![ProviderBean{ bean: bean.clone(), autowire_type: Some(dep_type.clone()) }]
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

    pub fn new_listable_bean_factory(beans_to_provide: &Vec<ProviderBean>, profile: &ProfileBuilder) -> TokenStream {
        let profile_name_str = profile.profile.clone();

        let profile_name = Ident::new(profile_name_str.as_str(), Span::call_site());

        let (singleton_idents, singleton_types,
            mutable_types, mutable_idents,
            abstract_mutable_paths, abstract_mutable_concrete,
            abstract_paths, abstract_paths_concrete)
            = Self::get_fill_ident_paths(beans_to_provide);

        abstract_paths.iter().for_each(|a_path| {
            log_message!("{} is the abstract path to create.", SynHelper::get_str(&a_path));
        });
        abstract_mutable_paths.iter().for_each(|a_path| {
            log_message!("{} is the abstract mutable path to create.", SynHelper::get_str(&a_path));
        });

        // TODO: if a bean is mutable it cannot have a reference not through mutex, so there will have
        //  to be two. However, for the abstract, the reference to the concrete type can be added in the
        //  map, so that the same bean is used.
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
                        let next_bean_definition = <dyn BeanFactory<#singleton_idents, #profile_name, U = #singleton_idents>>::get_bean(&listable_bean_factory);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn BeanFactory<#singleton_types, #profile_name, U = #singleton_idents>>::get_bean(&listable_bean_factory);
                        listable_bean_factory.add_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn MutableBeanFactory<Mutex<#mutable_idents>, #profile_name>>::get_bean(&listable_bean_factory);
                        listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn MutableBeanFactory<Mutex<#mutable_types>, #profile_name>>::get_bean(&listable_bean_factory);
                        listable_bean_factory.add_mutable_bean_definition(next_bean_definition);
                    )*
                    #(
                        let next_bean_definition = <dyn MutableBeanFactory<Mutex<dyn #abstract_mutable_paths>, #profile_name, U = #abstract_mutable_concrete>>::get_bean(&listable_bean_factory);
                        let type_id = TypeId::of::<Arc<Mutex<Box<dyn #abstract_mutable_paths>>>>();
                        listable_bean_factory.add_mutable_bean_definition_type_id(next_bean_definition, type_id);
                    )*
                    #(
                        let next_bean_definition = <dyn MutableBeanFactory<Mutex<Box<dyn #abstract_paths>>, #profile_name, U = Mutex<Box<dyn #abstract_paths>>>>::get_bean(&listable_bean_factory);
                        let type_id = TypeId::of::<Arc<Mutex<Box<dyn #abstract_paths>>>>();
                        listable_bean_factory.add_mutable_bean_definition_type_id(next_bean_definition, type_id);
                    )*
                    #(
                        let next_bean_definition = <dyn BeanFactory<dyn #abstract_paths, #profile_name, U = #abstract_paths_concrete>>::get_bean(&listable_bean_factory);
                        let type_id = TypeId::of::<Arc<dyn #abstract_paths>>();
                        listable_bean_factory.add_bean_definition_type_id(next_bean_definition, type_id);
                    )*

                    listable_bean_factory
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

    fn get_fill_ident_paths(beans_to_provide: &Vec<ProviderBean>)
        -> (Vec<Ident>, Vec<Type>,
            Vec<Type>, Vec<Ident>,
            Vec<Type>, Vec<Type>,
            Vec<Type>, Vec<Type>
        ) {

        let mut singleton_idents = vec![];
        let mut singleton_types = vec![];
        let mut mutable_types = vec![];
        let mut mutable_idents = vec![];
        let mut abstract_mutable_paths: Vec<Type> = vec![];
        let mut abstract_mutable_concrete: Vec<Type> = vec![];
        let mut abstract_paths: Vec<Type> = vec![];
        let mut abstract_paths_concrete: Vec<Type> = vec![];

        for provider_bean in beans_to_provide.iter() {

            let bean = &provider_bean.bean;

            // TODO: factory_fn
            if bean.factory_fn.is_some() && bean.struct_type.is_none() {
                continue;
            }

            Self::add_provider_bean_to_vec(
                &mut singleton_idents, &mut singleton_types,
                &mut mutable_types, &mut mutable_idents,
                &mut abstract_mutable_paths, &mut abstract_mutable_concrete,
                &mut abstract_paths, &mut abstract_paths_concrete,
                provider_bean, &bean
            );
        }

        (singleton_idents, singleton_types,
         mutable_types, mutable_idents,
         abstract_mutable_paths, abstract_mutable_concrete,
         abstract_paths, abstract_paths_concrete)
    }

    /// Add the bean (with abstraction if has one) to the vecs for the constructor of the container.
    fn add_provider_bean_to_vec(mut singleton_idents: &mut Vec<Ident>, mut singleton_types: &mut Vec<Type>, mut mutable_types: &mut Vec<Type>, mut mutable_idents: &mut Vec<Ident>, mut abstract_mutable_paths: &mut Vec<Type>, mut abstract_mutable_concrete: &mut Vec<Type>, mut abstract_paths: &mut Vec<Type>, mut abstract_paths_concrete: &mut Vec<Type>, provider_bean: &ProviderBean, bean: &&BeanDefinition) {
        provider_bean.autowire_type.as_ref()
            .map(|autowire_type| {
                Self::add_abstract_bean_type(
                    &mut abstract_mutable_paths, &mut abstract_mutable_concrete,
                    &mut abstract_paths, &mut abstract_paths_concrete,
                    bean, autowire_type
                );
            })
            .or_else(|| {
                Self::add_concrete_bean_type(
                    &mut singleton_idents, &mut singleton_types,
                    &mut mutable_types, &mut mutable_idents,
                    &bean
                )
            });
    }

    fn add_concrete_bean_type(mut singleton_idents: &mut Vec<Ident>, mut singleton_types: &mut Vec<Type>, mut mutable_types: &mut Vec<Type>, mut mutable_idents: &mut Vec<Ident>, bean: &&BeanDefinition) -> Option<()> {
        bean.bean_type.as_ref().map(|bean_type| {
            log_message!("Found bean type {:?}.", bean_type);
            match bean_type {
                BeanType::Singleton => {
                    log_message!("adding bean dep impl with type {} as singleton!", bean.id.clone());
                    Self::add_to_ident_type(
                        &mut singleton_idents, &mut singleton_types,
                        &mut mutable_types, &mut mutable_idents,
                        &bean
                    );
                }
                BeanType::Prototype => {
                    log_message!("Ignoring prototype bean {} when building bean factory.", bean.id.as_str());
                }
            };
        });
        None
    }

    fn add_abstract_bean_type(mut abstract_mutable_paths: &mut Vec<Type>, mut abstract_mutable_concrete: &mut Vec<Type>, mut abstract_paths: &mut Vec<Type>, mut abstract_paths_concrete: &mut Vec<Type>, bean: &BeanDefinition, autowire_type: &DependencyDescriptor) {
        bean.bean_type.as_ref().map(|bean_type| {
            log_message!("Found bean type {:?}.", bean_type);
            match bean_type {
                BeanType::Singleton => {
                    log_message!("adding bean dep impl with type {} as singleton!", bean.id.clone());
                    Self::add_abstract_trait_paths_for_bean(
                        &mut abstract_mutable_paths, &mut abstract_mutable_concrete,
                        &mut abstract_paths, &mut abstract_paths_concrete,
                        bean, autowire_type
                    );
                }
                BeanType::Prototype => {
                    log_message!("Ignoring prototype bean {} when building bean factory.", bean.id.as_str());
                }
            };
        });
    }

    fn add_abstract_trait_paths_for_bean(
        mut abstract_mutable_paths: &mut Vec<Type>,
        mut abstract_mutable_concrete: &mut Vec<Type>,
        mut abstract_paths: &mut Vec<Type>,
        mut abstract_paths_concrete: &mut Vec<Type>,
        bean: &BeanDefinition,
        autowire_type: &DependencyDescriptor
    ) {
        if bean.mutable {
            Self::add_abstract_concrete_path(abstract_mutable_paths, abstract_mutable_concrete, bean, autowire_type);
        } else {
            Self::add_abstract_concrete_path(abstract_paths, abstract_paths_concrete, bean, autowire_type);
        }
    }

    fn add_abstract_concrete_path(
        mut abstract_mutable_paths: &mut Vec<Type>,
        mut abstract_mutable_concrete: &mut Vec<Type>,
        bean: &BeanDefinition,
        autowire_type: &DependencyDescriptor
    ) {
        BeanFactoryInfo::get_abstract_type(autowire_type).as_ref()
            .map(|t| {
                bean.struct_type.as_ref().map(|struct_type| {
                    log_message!("Adding abstract mutable path: {} to struct {}.", SynHelper::get_str(&t), SynHelper::get_str(&struct_type));
                    abstract_mutable_paths.push(t.clone());
                    abstract_mutable_concrete.push(struct_type.clone());
                });
            });
    }

    fn add_to_ident_type(mut singleton_idents: &mut Vec<Ident>,
                         mut singleton_types: &mut Vec<Type>,
                         mut mutable_types: &mut Vec<Type>,
                         mut mutable_idents: &mut Vec<Ident>,
                         bean: &BeanDefinition) {
        if bean.mutable {
            Self::add_to(&mut mutable_idents, &mut mutable_types, &bean);
        } else {
            Self::add_to(&mut singleton_idents, &mut singleton_types, &bean);
        }
    }
}