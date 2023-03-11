use std::any::{Any, TypeId};
use std::collections::HashMap;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::__private::TokenStream2;
use syn::Type;
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::module_tree::{BeanType, Bean, Profile, BeanDefinitionType, AutowireType};

use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::AspectParser;
use crate::module_macro_lib::knockoff_context_builder::aspect_generator::AspectGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::{BeanFactoryGenerator, FactoryBeanBeanFactoryGenerator};
use crate::module_macro_lib::knockoff_context_builder::factory_generator::{FactoryGen, FactoryGenerator};
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::profile_tree::ProfileTree;

pub mod factory_generator;
pub mod bean_factory_generator;
pub mod token_stream_generator;
pub mod aspect_generator;

#[derive(Default)]
pub struct ApplicationContextGenerator {
    factory_generators: Vec<Box<dyn TokenStreamGenerator>>,
    bean_factory_generators: Vec<Box<dyn TokenStreamGenerator>>,
    aspect_generators: Vec<Box<dyn TokenStreamGenerator>>,
    profiles: Vec<Profile>,
}

impl TokenStreamGenerator for ApplicationContextGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        let mut ts = TokenStream::default();
        ts.append_all(Self::context_imports());
        ts.append_all(ApplicationContextGenerator::init_bean_factory());
        ts.append_all(FactoryGen::impl_listable_factory());
        self.write_generators(&mut ts);
        ts.append_all(
            Self::finish_abstract_factory(
            self.profiles.iter()
                    .map(|p| p.profile.clone())
                    .collect()
        ));
        ts
    }
}

impl ApplicationContextGenerator {

    pub fn create_context_generator(profile_tree: &ProfileTree) -> Self {
        let factory_generators = profile_tree.injectable_types.iter()
            .flat_map(|bean_def_type_profile| Self::create_factory_generators(&bean_def_type_profile))
            .collect::<Vec<Box<dyn TokenStreamGenerator>>>();
        let bean_factory_generators = Self::create_bean_factory_generators(&profile_tree.injectable_types);
        let profiles = profile_tree.injectable_types.keys()
            .map(|p| p.clone())
            .collect();
        let aspect_generators = vec![Self::create_aspect_generator(&profile_tree)];
        Self {
            factory_generators,
            bean_factory_generators,
            profiles,
            aspect_generators
        }
    }

    fn create_aspect_generator(from: &ProfileTree) -> Box<dyn TokenStreamGenerator> {
        Box::new(AspectGenerator::new(from))
    }

    fn create_factory_generators(from: &(&Profile, &Vec<BeanDefinitionType>)) -> Vec<Box<dyn TokenStreamGenerator>> {
        vec![
            FactoryGen::new_factory_generator(from.0.clone(), from.1.clone())
        ]
    }

    fn create_bean_factory_generators(from: &HashMap<Profile, Vec<BeanDefinitionType>>) -> Vec<Box<dyn TokenStreamGenerator>> {
        FactoryBeanBeanFactoryGenerator::new_bean_factory_generators(
            &Self::get_concrete_beans(&from),
            &Self::get_abstract_beans(&from)
        )
    }

    fn get_abstract_beans(from: &HashMap<Profile, Vec<BeanDefinitionType>>) -> Vec<(Bean, AutowireType, Profile)> {
        from.iter().flat_map(|f|
            f.1.iter()
            .flat_map(|b| {
                match b {
                    BeanDefinitionType::Abstract { bean, dep_type } => {
                        dep_type.item_impl.trait_.as_ref().map(|trait_type| {
                            log_message!("Found abstract bean with id: {} and abstract type: {}.",
                            &bean.id, SynHelper::get_str(&dep_type.item_impl.trait_.as_ref().unwrap().1));
                            vec![(bean.to_owned(), dep_type.to_owned(), f.0.clone())]
                        })
                            .or(Some(vec![]))
                            .unwrap()
                    }
                    BeanDefinitionType::Concrete { .. } => {
                        vec![]
                    }
                }
            })
        ).collect()
    }

    fn get_concrete_beans(from: &HashMap<Profile, Vec<BeanDefinitionType>>) -> Vec<Bean> {
        from.iter().flat_map(|f| f.1)
            .flat_map(|b| {
                match b {
                    BeanDefinitionType::Abstract { .. } => {
                        vec![]
                    }
                    BeanDefinitionType::Concrete { bean } => {
                        vec![(bean.id.clone(), bean.clone())]
                    }
                }
            })
            .collect::<HashMap<String, Bean>>()
            .values()
            .map(|b| b.to_owned())
            .collect()
    }


    pub fn context_imports() -> TokenStream {
        let ts = quote! {
            use module_macro_lib::module_macro_lib::knockoff_context::{AbstractListableFactory, ApplicationContext, Profile, ContainsBeans};
            use std::sync::Mutex;
            use paste::paste;
        };
        ts.into()
    }

    pub fn init_bean_factory() -> TokenStream {
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
                    self.inner.type_id().clone()
                }
            }

            pub trait BeanFactory<T: 'static + Send + Sync + ?Sized, P: Profile> {
                type U;
                fn get_bean(&self) -> BeanDefinition<Self::U>;
            }

            pub trait BeanContainer<T: 'static + Send + Sync + ?Sized> {
                type U;
                fn fetch_bean(&self) -> Option<Arc<Self::U>>;
            }

            pub trait BeanContainerProfile<T: 'static + Send + Sync + ?Sized, P: Profile> {
                type U;
                fn fetch_bean_profile(&self) -> Option<Arc<Self::U>>;
            }

            pub trait PrototypeBeanFactory<T: 'static + Send + Sync + ?Sized, P: Profile> {
                fn get_bean(&self) -> PrototypeBeanDefinition<T>;
            }

            pub trait MutableBeanFactory<T: 'static + Send + Sync + ?Sized, P: Profile> {
                type U;
                fn get_bean(&self) -> MutableBeanDefinition<Self::U>;
            }

            pub trait MutableFactoryBean<T: 'static + Send + Sync + ?Sized, P: Profile> {
                type U;
                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> MutableBeanDefinition<Self::U>;
                fn get_bean_type_id(&self) -> TypeId;
                fn is_singleton() -> bool;
            }

            pub trait FactoryBean<T: 'static + Send + Sync + ?Sized, P: Profile> {
                type U;
                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<Self::U>;
                fn get_bean_type_id(&self) -> TypeId;
                fn is_singleton() -> bool;
            }

            pub trait PrototypeFactoryBean<T: 'static + Send + Sync + ?Sized, P: Profile> {
                fn get_prototype_bean(&self) -> PrototypeBeanDefinition<T>;
                fn get_bean_type_id() -> TypeId;
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
                    let factory = self.factories.get(&P::name())
                        .unwrap();

                    factory.singleton_bean_definitions.get(&type_id)
                        .map(|bean_def| bean_def.inner.clone().downcast::<T>().ok())
                        .flatten()
                        .or_else(|| factory.mutable_bean_definitions.get(&type_id)
                                .map(|bean_def| bean_def.inner.clone().downcast::<T>().ok())
                                .flatten()
                        )
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

                fn get_beans(&self) -> Vec<Arc<dyn Any + Send + Sync>>
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

    fn write_generators(&self, mut ts: &mut TokenStream) {
        self.bean_factory_generators.iter()
            .for_each(|factory_gens|
                ts.append_all(factory_gens.generate_token_stream())
            );
        self.factory_generators.iter()
            .for_each(|factory_gens|
                ts.append_all(factory_gens.generate_token_stream())
            );
        self.aspect_generators.iter()
            .for_each(|aspect_generator|
                ts.append_all(aspect_generator.generate_token_stream())
            );
    }
}



