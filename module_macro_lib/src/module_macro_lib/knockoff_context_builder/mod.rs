use std::any::{Any, TypeId};
use std::collections::HashMap;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::__private::TokenStream2;
use syn::Type;
use bean_factory_generator::factory_factory_generator::FactoryBeanBeanFactoryGenerator;
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType, BeanType};

use knockoff_logging::{initialize_log, use_logging};
use module_macro_codegen::aspect::AspectParser;
use module_macro_shared::dependency::DependencyDescriptor;
use module_macro_shared::profile_tree::ProfileBuilder;
use crate::module_macro_lib::knockoff_context_builder::aspect_generator::AspectGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_constructor_generator::BeanConstructorGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_generator::BeanFactoryGenerator;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::{AbstractBeanFactoryInfo, BeanFactoryInfo, BeanFactoryInfoFactory, ConcreteBeanFactoryInfo};
use crate::module_macro_lib::knockoff_context_builder::factory_generator::{FactoryGen, FactoryGenerator};
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::{TokenStreamGenerator, UserProvidedTokenStreamGenerator};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use module_macro_shared::profile_tree::ProfileTree;

pub mod factory_generator;
pub mod bean_factory_generator;
pub mod token_stream_generator;
pub mod aspect_generator;
pub mod bean_factory_info;
pub mod bean_constructor_generator;

#[derive(Default)]
pub struct ApplicationContextGenerator {
    factory_generators: Vec<Box<dyn TokenStreamGenerator>>,
    bean_factory_generators: Vec<Box<dyn TokenStreamGenerator>>,
    aspect_generators: Vec<Box<dyn TokenStreamGenerator>>,
    constructor_generator: Vec<Box<dyn TokenStreamGenerator>>,
    dynamic_token_provider: Vec<Box<dyn TokenStreamGenerator>>,
    profiles: Vec<ProfileBuilder>,
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
        let (concrete_bean_factory_info, abstract_bean_factory_info) = Self::create_bean_factory_info(&profile_tree);

        let factory_generators = Self::factory_generators(profile_tree);
        let bean_factory_generators = Self::create_bean_factory_generators(
            &concrete_bean_factory_info, &abstract_bean_factory_info
        );
        let constructor_generator = vec![BeanConstructorGenerator::create_bean_constructor_generator(
            concrete_bean_factory_info.iter()
                .filter(|c| c.factory_fn.is_none())
                .map(|b| b.clone())
                .collect::<Vec<BeanFactoryInfo>>()
        )];

        let profiles = profile_tree.injectable_types.keys()
            .map(|p| p.clone())
            .collect();
        let aspect_generators = vec![Self::create_aspect_generator(&profile_tree)];
        let dynamic_token_provider = vec![Box::new(UserProvidedTokenStreamGenerator::new(&profile_tree)) as Box<dyn TokenStreamGenerator>];

        Self {
            factory_generators,
            bean_factory_generators,
            profiles,
            aspect_generators,
            constructor_generator,
            dynamic_token_provider
        }
    }

    /// Parse the ProfileTree into BeanFactoryInfo objects, which will then be used to generate the BeanFactories for each bean.
    fn create_bean_factory_info(profile_tree: &ProfileTree) -> (Vec<BeanFactoryInfo>, Vec<BeanFactoryInfo>) {
        let concrete_bean_factory_info = Self::get_concrete_beans(&profile_tree.injectable_types).iter()
            .flat_map(|b| ConcreteBeanFactoryInfo::create_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        let abstract_bean_factory_info = Self::get_abstract_beans(&profile_tree.injectable_types).iter()
            .flat_map(|b| AbstractBeanFactoryInfo::create_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        (concrete_bean_factory_info, abstract_bean_factory_info)
    }

    fn factory_generators(profile_tree: &ProfileTree) -> Vec<Box<dyn TokenStreamGenerator>> {
        profile_tree.injectable_types.iter()
            .flat_map(|bean_def_type_profile| Self::create_factory_generators(&bean_def_type_profile))
            .collect::<Vec<Box<dyn TokenStreamGenerator>>>()
    }

    fn create_aspect_generator(from: &ProfileTree) -> Box<dyn TokenStreamGenerator> {
        Box::new(AspectGenerator::new(from))
    }

    fn create_factory_generators(from: &(&ProfileBuilder, &Vec<BeanDefinitionType>)) -> Vec<Box<dyn TokenStreamGenerator>> {
        vec![
            FactoryGen::new_factory_generator(from.0.clone(), from.1.clone())
        ]
    }

    fn create_bean_factory_generators(concrete: &Vec<BeanFactoryInfo>, abstract_beans: &Vec<BeanFactoryInfo>) -> Vec<Box<dyn TokenStreamGenerator>> {
        FactoryBeanBeanFactoryGenerator::new_bean_factory_generators(
            concrete,
            abstract_beans,
        )
    }

    fn get_abstract_beans(from: &HashMap<ProfileBuilder, Vec<BeanDefinitionType>>) -> Vec<(BeanDefinition, DependencyDescriptor, ProfileBuilder)> {
        from.iter().flat_map(|f|
            f.1.iter()
            .flat_map(|b| {
                match b {
                    BeanDefinitionType::Abstract { bean, dep_type } => {
                        BeanFactoryInfo::get_abstract_type(dep_type)
                            .map(|_| {
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

    fn get_concrete_beans(from: &HashMap<ProfileBuilder, Vec<BeanDefinitionType>>) -> Vec<BeanDefinition> {
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
            .collect::<HashMap<String, BeanDefinition>>()
            .values()
            .map(|b| b.to_owned())
            .collect()
    }


    pub fn context_imports() -> TokenStream {
        let ts = quote! {
            use module_macro_lib::module_macro_lib::knockoff_context::{AbstractListableFactory, ApplicationContext, ContainsBeans, Profile};
            use module_macro_shared::profile_tree::ProfileBuilder;
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

            pub trait PrototypeBeanContainer<T: 'static + Send + Sync + ?Sized> {
                type U;
                fn fetch_bean(&self) -> Self::U;
            }

            pub trait PrototypeBeanContainerProfile<T: 'static + Send + Sync + ?Sized, P: Profile> {
                type U;
                fn fetch_bean_profile(&self) -> Self::U;
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

            pub trait PrototypeBeanFactory<T: ?Sized + Send + Sync, P: Profile> {
                type U;
                fn get_prototype_bean(listable_bean_factory: &ListableBeanFactory) -> Self::U;
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
                pub(crate) factories: HashMap<String,ListableBeanFactory>,
                pub(crate) profiles: Vec<String>,
                pub(crate) default_profile: String
            }

            impl AppCtx {
                fn get_bean_for_factory<T: Any + Send + Sync>(factory: &ListableBeanFactory) -> Option<Arc<T>> {
                    let type_id = TypeId::of::<Arc<T>>();
                    factory.singleton_bean_definitions.get(&type_id)
                        .map(|bean_def| bean_def.inner.clone().downcast::<T>().ok())
                        .flatten()
                        .or_else(|| factory.mutable_bean_definitions.get(&type_id)
                                .map(|bean_def| bean_def.inner.clone().downcast::<T>().ok())
                                .flatten()
                        )
                }
            }

            impl ApplicationContext for AppCtx {

                fn get_bean<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
                    self.factories.get(&self.default_profile)
                        .map(|factory| AppCtx::get_bean_for_factory::<T>(factory))
                        .flatten()
                }

                fn get_bean_for_profile<T: Any + Send + Sync, P: Profile>(&self) -> Option<Arc<T>> {
                    self.factories.get(&P::name())
                        .map(|factory| AppCtx::get_bean_for_factory::<T>(factory))
                        .flatten()
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
                        profiles,
                        default_profile: ProfileBuilder::default().profile
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
        self.constructor_generator.iter()
            .for_each(|constructor_gen|
                ts.append_all(constructor_gen.generate_token_stream())
            );
        self.dynamic_token_provider.iter()
            .for_each(|constructor_gen|
                ts.append_all(constructor_gen.generate_token_stream())
            );
    }
}



