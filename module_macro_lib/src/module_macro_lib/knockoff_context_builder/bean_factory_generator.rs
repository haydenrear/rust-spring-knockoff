use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::token::Mut;
use syn::{Path, Type};
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use crate::module_macro_lib::module_tree::{AutowiredField, AutowireType, Bean, BeanPath, DepType, Profile};
use knockoff_logging::{initialize_log, use_logging};

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;


#[derive(Clone, Default)]
pub struct BeanFactoryInfo {
    fields: Vec<FieldInfo>,
    mutable_fields: Vec<MutableFieldInfo>,
    abstract_fields: Vec<AbstractFieldInfo>,
    mutable_abstract_fields: Vec<MutableAbstractFieldInfo>,
    concrete_type: Option<Type>,
    abstract_type: Option<Path>,
    ident_type: Option<Ident>,
    profile: Option<Profile>
}

#[derive(Clone)]
pub struct FieldInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident
}

#[derive(Clone)]
pub struct MutableFieldInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident
}

#[derive(Clone)]
pub struct AbstractFieldInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident,
    autowire_type: AutowiredField,
    qualifier: Option<String>,
    profile: Option<String>
}

#[derive(Clone)]
pub struct MutableAbstractFieldInfo {
    field_type: Type,
    concrete_field_type: Type,
    field_ident: Ident,
    autowire_type: AutowiredField,
    qualifier: Option<String>,
    profile: Option<String>
}

pub trait BeanFactoryGenerator: TokenStreamGenerator {

    fn create_concrete_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream;

    fn create_abstract_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream;

    fn new_bean_factory_generators(concrete_beans: &Vec<Bean>, abstract_beans: &Vec<(Bean, AutowireType, Profile)>) -> Vec<Box<dyn TokenStreamGenerator>> {
        vec![
            Box::new(MutableBeanFactoryGenerator::new_bean_factory_generator(concrete_beans, abstract_beans)) as Box<dyn TokenStreamGenerator>,
            Box::new(FactoryBeanBeanFactoryGenerator::new_bean_factory_generator(concrete_beans, abstract_beans)) as Box<dyn TokenStreamGenerator>,
            Box::new(PrototypeBeanFactoryGenerator::new_bean_factory_generator(concrete_beans, abstract_beans)) as Box<dyn TokenStreamGenerator>,
        ]
    }

    fn create_concrete_bean_factory_info(bean: &Bean) -> BeanFactoryInfo {

        log_message!("Creating bean factory info for bean with id {} with has {} dependencies.", &bean.id, bean.deps_map.len());

        // TODO: Fix this - not adding dependencies in some cases.
        let mutable_fields = Self::get_mutable_singleton_field_ids(bean);
        let fields = Self::get_singleton_field_ids(bean);
        let mutable_abstract_fields = Self::get_abstract_mutable_field_ids(&bean);
        let abstract_fields = Self::get_abstract_field_ids(&bean);

        BeanFactoryInfo {
            fields,
            mutable_fields,
            abstract_fields,
            mutable_abstract_fields,
            concrete_type: bean.struct_type.clone(),
            abstract_type: None,
            ident_type: bean.ident.clone(),
            profile: None,
        }
    }

    fn create_abstract_bean_factory_info(bean_type: &(Bean, AutowireType, Profile)) -> BeanFactoryInfo {

        let bean = bean_type.0.to_owned();
        let abstract_type = bean_type.1.to_owned().item_impl.trait_.map(|t| t.1);
        let mutable_fields = Self::get_mutable_singleton_field_ids(&bean);
        let fields = Self::get_singleton_field_ids(&bean);
        let mutable_abstract_fields = Self::get_abstract_mutable_field_ids(&bean);
        let abstract_fields = Self::get_abstract_field_ids(&bean);

        BeanFactoryInfo {
            fields,
            mutable_fields,
            abstract_fields,
            mutable_abstract_fields,
            concrete_type: bean.struct_type.clone(),
            abstract_type,
            ident_type: bean.ident.clone(),
            profile: Some(bean_type.2.to_owned())
        }
    }

    fn get_singleton_field_ids(bean: &Bean) -> Vec<FieldInfo> {
        Self::get_field_ids::<FieldInfo>(bean, &Self::create_dep_type)
    }

    fn get_abstract_field_ids(bean: &Bean) -> Vec<AbstractFieldInfo> {
        Self::get_field_ids::<AbstractFieldInfo>(bean, &Self::create_abstract_dep_type)
    }

    fn get_abstract_mutable_field_ids(bean: &Bean) -> Vec<MutableAbstractFieldInfo> {
        Self::get_field_ids::<MutableAbstractFieldInfo>(
            bean,
            &Self::create_mutable_abstract_dep_type
        )
    }

    fn create_mutable_dep_type(dep_type: &DepType) -> Option<MutableFieldInfo> {
        if dep_type.is_abstract.is_some() && *dep_type.is_abstract.as_ref().unwrap() {
            return None;
        }
        dep_type.bean_type_path
            .as_ref()
            .filter(|d| d.is_mutable())
            .map(|type_path| type_path.get_autowirable_type())
            .flatten()
            .map(|field_type| MutableFieldInfo {
                concrete_field_type: dep_type.bean_info.concrete_type_of_field_bean_type.clone().or(Some(field_type.clone())).unwrap(),
                field_type,
                field_ident: dep_type.bean_info.field.ident.clone().unwrap(),
            })
    }

    fn create_dep_type(dep_type: &DepType) -> Option<FieldInfo> {
        if dep_type.is_abstract.is_some() && *dep_type.is_abstract.as_ref().unwrap() {
            return None;
        }
        dep_type.bean_type_path
            .as_ref()
            .filter(|d| d.is_not_mutable())
            .map(|type_path| type_path.get_autowirable_type())
            .flatten()
            .map(|field_type| FieldInfo {
                concrete_field_type: dep_type.bean_info.concrete_type_of_field_bean_type.clone().or(Some(field_type.clone())).unwrap(),
                field_type,
                field_ident: dep_type.bean_info.field.ident.clone().unwrap(),
            })
    }

    fn create_abstract_dep_type(dep_type: &DepType) -> Option<AbstractFieldInfo> {
        if dep_type.is_abstract.is_some() && *dep_type.is_abstract.as_ref().unwrap() {
            return dep_type.bean_type_path
                .as_ref()
                .filter(|d| d.is_not_mutable())
                .map(|type_path| type_path.get_autowirable_type())
                .flatten()
                .map(|field_type| AbstractFieldInfo {
                    concrete_field_type: dep_type.bean_info.concrete_type_of_field_bean_type.clone().or(Some(field_type.clone())).unwrap(),
                    field_type,
                    autowire_type: dep_type.bean_info.clone(),
                    qualifier: dep_type.bean_info.qualifier.clone(),
                    profile: None,
                    field_ident: dep_type.bean_info.field.ident.clone().unwrap(),
                });
        }
        None
    }

    fn create_mutable_abstract_dep_type(dep_type: &DepType) -> Option<MutableAbstractFieldInfo> {
        if dep_type.is_abstract.is_some() && *dep_type.is_abstract.as_ref().unwrap() {
            return dep_type.bean_type_path
                .as_ref()
                .filter(|d| d.is_mutable())
                .map(|type_path| type_path.get_autowirable_type())
                .flatten()
                .map(|field_type| MutableAbstractFieldInfo {
                    concrete_field_type: dep_type.bean_info.concrete_type_of_field_bean_type.clone().or(Some(field_type.clone())).unwrap(),
                    field_type,
                    autowire_type: dep_type.bean_info.clone(),
                    qualifier: dep_type.bean_info.qualifier.clone(),
                    profile: None,
                    field_ident: dep_type.bean_info.field.ident.clone().unwrap(),
                });
        }
        None
    }

    fn get_field_ids<T>(
        token_type: &Bean,
        creator: &dyn Fn(&DepType) -> Option<T>
    ) -> Vec<T> {
        let field_types = token_type.deps_map
            .clone().iter()
            .flat_map(|d| creator(d)
                    .map(|item| vec![item])
                    .or(Some(vec![]))
                    .unwrap()
            )
            .collect::<Vec<T>>();

        field_types
    }

    fn get_mutable_singleton_field_ids(token_type: &Bean) -> Vec<MutableFieldInfo> {
        Self::get_field_ids::<MutableFieldInfo>(token_type, &Self::create_mutable_dep_type)
    }

    fn bean_dep_impl_not_abstract(b: &DepType) -> bool {
        b.is_abstract.is_some() && !*b.is_abstract.as_ref().unwrap()
    }

    fn bean_dep_impl_abstract(b: &DepType) -> bool {
        !Self::bean_dep_impl_not_abstract(b)
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

    fn get_field_types(b: &BeanFactoryInfo) -> (Vec<Type>, Vec<Ident>, Vec<Type>, Vec<Ident>, Vec<Type>, Vec<Type>, Vec<Ident>, Vec<Type>, Vec<Type>, Vec<Ident>, Vec<Type>, Vec<Type>) {
        let field_types = b.fields.iter()
            .map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let field_idents = b.fields.iter()
            .map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let field_concrete = b.fields.iter()
            .map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_field_idents = b.mutable_fields.iter().map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let mutable_field_types = b.mutable_fields.iter().map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_field_concrete = b.mutable_fields.iter().map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let abstract_field_ident = b.abstract_fields.iter().map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let abstract_field_types = b.abstract_fields.iter().map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let abstract_field_concrete = b.abstract_fields.iter().map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_abstract_field_ident = b.mutable_abstract_fields.iter().map(|f| f.field_ident.clone())
            .collect::<Vec<Ident>>();
        let mutable_abstract_field_types = b.mutable_abstract_fields.iter().map(|f| f.field_type.clone())
            .collect::<Vec<Type>>();
        let mutable_abstract_field_concrete = b.mutable_abstract_fields.iter().map(|f| f.concrete_field_type.clone())
            .collect::<Vec<Type>>();
        (field_types, field_idents, field_concrete,
         mutable_field_idents, mutable_field_types, mutable_field_concrete,
         abstract_field_ident, abstract_field_types, abstract_field_concrete,
         mutable_abstract_field_ident, mutable_abstract_field_types, mutable_abstract_field_concrete)
    }


    fn new_bean_factory_generator(beans: &Vec<Bean>, abstract_beans: &Vec<(Bean, AutowireType, Profile)>) -> Self where Self: Sized {
        let bean_factories_to_implement = beans.iter()
            .map(|b| Self::create_concrete_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        let abstract_bean_factories = abstract_beans.iter()
            .map(|b| Self::create_abstract_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        Self::new(bean_factories_to_implement, abstract_bean_factories)
    }

    fn get_concrete_factories(&self) -> Vec<BeanFactoryInfo>;

    fn get_abstract_factories(&self) -> Vec<BeanFactoryInfo>;

    fn new(beans: Vec<BeanFactoryInfo>, abstract_beans: Vec<BeanFactoryInfo>) -> Self;
}

pub struct MutableBeanFactoryGenerator {
    bean_factories_to_implement: Vec<BeanFactoryInfo>,
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

        let (field_types, field_idents, concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_identifiers, abstract_mutable_field_types, concrete_mutable_abstract)
            = Self::get_field_types(bean_factory_info);

        log_message!("Creating concrete mutable factory for: {}.", SynHelper::get_str(&bean_factory_info.concrete_type.as_ref().unwrap()));

        log_message!("{} is number of field idents, {} is number of field types.", field_types.len(), field_idents.len());
        log_message!("{} is number of mutable idents, {} is number of mutable field types.", mutable_identifiers.len(), mutable_field_types.len());
        log_message!("{} is number of abstract idents, {} is number of abstract field types.", abstract_field_idents.len(), abstract_field_types.len());
        log_message!("{} is number of abstract mutable idents, {} is number of abstract mutable field types.", abstract_mutable_identifiers.len(), abstract_mutable_field_types.len());

        let struct_type: Ident = bean_factory_info.concrete_type
            .as_ref()
            .map(|t| Ident::new(t.to_token_stream().to_string().as_str(), Span::call_site()))
            .or(bean_factory_info.ident_type.clone())
            .unwrap();

        // let abstract_type = bean_factory_info.abstract_type.as_ref().unwrap();
        let default_profile = Ident::new(Profile::default().profile.as_str(), Span::call_site());
        let injectable_code = quote! {

                impl MutableBeanFactory<Mutex<#struct_type>, #default_profile> for ListableBeanFactory {
                    type U = Mutex<#struct_type>;
                    fn get_bean(&self) -> MutableBeanDefinition<Mutex<#struct_type>> {
                        let this_component = <MutableBeanDefinition<Mutex<#struct_type>>>::get_bean(&self);
                        this_component
                    }
                }

                impl BeanContainer<Mutex<#struct_type>> for ListableBeanFactory {
                    type U = Mutex<#struct_type>;
                    fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                        self.mutable_bean_definitions.get(&TypeId::of::<Arc<Mutex<#struct_type>>>())
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

                impl BeanContainerProfile<Mutex<#struct_type>, #default_profile> for ListableBeanFactory {
                    type U = Mutex<#struct_type>;
                    fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                        self.mutable_bean_definitions.get(&TypeId::of::<Arc<Mutex<#struct_type>>>())
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

                impl MutableFactoryBean<Mutex<#struct_type>, #default_profile> for MutableBeanDefinition<Mutex<#struct_type>> {
                    type U = Mutex<#struct_type>;
                    fn get_bean(listable_bean_factory: &ListableBeanFactory) -> MutableBeanDefinition<Mutex<#struct_type>> {
                        let mut inner = #struct_type::default();
                        #(
                            let bean_def: BeanDefinition<#field_types> = <ListableBeanFactory as BeanFactory<#field_types, #default_profile>>::get_bean(listable_bean_factory);
                            let arc_bean_def: Arc<#field_types> = bean_def.inner;
                            inner.#field_idents = arc_bean_def.clone();
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>>
                                = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>, #default_profile>>::get_bean(
                                    listable_bean_factory
                                );
                            let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                            inner.#mutable_identifiers = arc_bean_def.clone();
                        )*
                        #(
                            let arc_bean_def = <ListableBeanFactory as BeanFactory<#abstract_field_types, #default_profile>>::get_bean(listable_bean_factory)
                                        .inner.clone();
                            inner.#abstract_field_idents = arc_bean_def;
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<dyn #abstract_mutable_field_types>>
                                = <ListableBeanFactory as MutableBeanFactory<Mutex<dyn #abstract_mutable_field_types>, #default_profile>>::get_bean(
                                    listable_bean_factory
                                );
                            let arc_bean_def: Arc<Mutex<Box<dyn #abstract_mutable_field_types>>> = bean_def.inner;
                            inner.#abstract_mutable_identifiers = arc_bean_def.clone();
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

    fn create_abstract_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream {

        let profile_ident = Ident::new(bean_factory_info.profile.as_ref().unwrap().profile.as_str(), Span::call_site());

        let (field_types, field_idents, concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_identifiers, abstract_mutable_field_types, concrete_mutable_abstract)
            = Self::get_field_types(bean_factory_info);

        log_message!("Building container");

        let concrete_type: Ident = bean_factory_info.concrete_type
            .as_ref()
            .map(|t| Ident::new(t.to_token_stream().to_string().as_str(), Span::call_site()))
            .or(bean_factory_info.ident_type.clone())
            .unwrap();

        let abstract_type = bean_factory_info.abstract_type.as_ref().unwrap();

        /// If you implement the dyn factory by returning the concrete type, then you save the bean
        /// as the concrete type. You can then implement the same way to get the type id easily.
        let injectable_code = quote! {

            impl MutableBeanFactory<Mutex<Box<dyn #abstract_type>>, #profile_ident> for ListableBeanFactory {
                type U = Mutex<Box<#concrete_type>>;
                fn get_bean(&self) -> MutableBeanDefinition<Self::U> {
                    <MutableBeanDefinition<Mutex<Box<dyn #abstract_type>>>>::get_bean(&self)
                }
            }

            impl BeanContainer<Mutex<Box<dyn #abstract_type>>> for ListableBeanFactory {
                type U = Mutex<Box<#concrete_type>>;
                fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                    self.mutable_bean_definitions.get(&TypeId::of::<Arc<Mutex<Box<dyn #abstract_type>>>>())
                        .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                        .flatten()
                }
            }

            impl BeanContainerProfile<Mutex<Box<dyn #abstract_type>>, #profile_ident> for ListableBeanFactory {
                type U = Mutex<Box<#concrete_type>>;
                fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                    self.mutable_bean_definitions.get(&TypeId::of::<Mutex<Arc<Box<dyn #abstract_type>>>>())
                        .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                        .flatten()
                }
            }

            impl MutableFactoryBean<Mutex<Box<dyn #abstract_type>>, #profile_ident> for MutableBeanDefinition<Mutex<Box<dyn #abstract_type>>> {
                type U = Mutex<Box<#concrete_type>>;
                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> MutableBeanDefinition<Self::U> {
                    let mut inner = #concrete_type::default();
                    #(
                        let bean_def: BeanDefinition<#field_types>
                            = <ListableBeanFactory as BeanFactory<#field_types, #profile_ident>>::get_bean(listable_bean_factory);
                        let arc_bean_def: Arc<#field_types> = bean_def.inner;
                        inner.#field_idents = arc_bean_def.clone();
                    )*
                    #(
                        let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>>
                            = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>, #profile_ident>>::get_bean(
                                listable_bean_factory
                            );
                        let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                        inner.#mutable_identifiers = arc_bean_def.clone();
                    )*
                    #(
                        let bean_def = <ListableBeanFactory as BeanFactory<#abstract_field_types, #profile_ident>>::get_bean(listable_bean_factory);
                        inner.#abstract_field_idents = arc_bean_def.inner.clone();
                    )*
                    #(
                        let bean_def: MutableBeanDefinition<Mutex<Box<dyn #abstract_mutable_field_types>>>
                            = <ListableBeanFactory as MutableBeanFactory<Mutex<Box<dyn #abstract_mutable_field_types>, #profile_ident>>::get_bean(
                                listable_bean_factory
                            );
                        let arc_bean_def: Arc<Mutex<Box<dyn #abstract_mutable_field_types>>> = bean_def.inner;
                        inner.#abstract_mutable_identifiers = arc_bean_def.clone();
                    )*
                    let m = MutableBeanDefinition {
                        inner: Arc::new(Mutex::new(Box::new(inner)))
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
        self.bean_factories_to_implement.clone()
    }

    fn get_abstract_factories(&self) -> Vec<BeanFactoryInfo> {
        self.abstract_bean_factories.clone()
    }

    fn new(bean_factories_to_implement: Vec<BeanFactoryInfo>, abstract_bean_factories: Vec<BeanFactoryInfo>) -> Self {
        Self {
            bean_factories_to_implement,
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

        let (field_types, field_idents, concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_idents, abstract_mutable_field_types, concrete_mutable_abstract)
            = Self::get_field_types(bean_factory_info);


        log_message!("Creating concrete factory for: {}.", SynHelper::get_str(&bean_factory_info.concrete_type.as_ref().unwrap()));

        log_message!("{} is number of field idents, {} is number of field types.", field_types.len(), field_idents.len());
        log_message!("{} is number of mutable idents, {} is number of mutable field types.", mutable_identifiers.len(), mutable_field_types.len());
        log_message!("{} is number of abstract idents, {} is number of abstract field types.", abstract_field_idents.len(), abstract_field_types.len());
        log_message!("{} is number of abstract mutable idents, {} is number of abstract mutable field types.", abstract_mutable_idents.len(), abstract_mutable_field_types.len());

        let struct_type: Ident = bean_factory_info.concrete_type
            .as_ref()
            .map(|t| Ident::new(t.to_token_stream().to_string().as_str(), Span::call_site()))
            .or(bean_factory_info.ident_type.clone())
            .unwrap();

        let default_profile = Ident::new(Profile::default().profile.as_str(), Span::call_site());

        let injectable_code = quote! {

                impl BeanFactory<#struct_type, #default_profile> for ListableBeanFactory {
                    type U = #struct_type;
                    fn get_bean(&self) -> BeanDefinition<#struct_type> {
                        let this_component = <BeanDefinition<#struct_type>>::get_bean(&self);
                        this_component
                    }
                }


                impl BeanContainer<#struct_type> for ListableBeanFactory {
                    type U = #struct_type;
                    fn fetch_bean(&self) -> Option<Arc<Self::U>> {
                        self.singleton_bean_definitions.get(&TypeId::of::<Arc<#struct_type>>())
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

                impl BeanContainerProfile<#struct_type, #default_profile> for ListableBeanFactory {
                    type U = Mutex<#struct_type>;
                    fn fetch_bean_profile(&self) -> Option<Arc<Self::U>> {
                        self.singleton_bean_definitions.get(&TypeId::of::<Arc<#struct_type>>())
                            .map(|s| s.inner.clone().downcast::<Self::U>().ok())
                            .flatten()
                    }
                }

                impl FactoryBean<#struct_type, #default_profile> for BeanDefinition<#struct_type> {
                    type U = #struct_type;
                    fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<#struct_type> {
                        let mut inner = #struct_type::default();
                        #(
                            let bean_def: BeanDefinition<#field_types> = <ListableBeanFactory as BeanFactory<#field_types, #default_profile>>::get_bean(listable_bean_factory);
                            let arc_bean_def: Arc<#field_types> = bean_def.inner;
                            inner.#field_idents = arc_bean_def.clone();
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>>
                                = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>, #default_profile>>::get_bean(listable_bean_factory);
                            let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                            inner.#mutable_identifiers = arc_bean_def.clone();
                        )*
                        #(
                            let arc_bean_def = <ListableBeanFactory as BeanFactory<#abstract_field_types, #default_profile>>::get_bean(listable_bean_factory);
                            inner.#abstract_field_idents = arc_bean_def.inner.clone();
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<dyn #abstract_mutable_field_types>>
                                = <ListableBeanFactory as MutableBeanFactory<Mutex<dyn #abstract_mutable_field_types>, #default_profile>>::get_bean(
                                    listable_bean_factory
                                );
                            let arc_bean_def: Arc<Mutex<dyn #abstract_mutable_field_types>> = bean_def.inner;
                            inner.#abstract_mutable_idents = arc_bean_def.clone();
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

    fn create_abstract_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream {

        let (field_types, field_idents, concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_idents, abstract_mutable_field_types, concrete_mutable_abstract)
            = Self::get_field_types(bean_factory_info);

        log_message!("Building factory generator for {}", SynHelper::get_str(&bean_factory_info.abstract_type.as_ref().unwrap()));

        let struct_type: Ident = bean_factory_info.concrete_type
            .as_ref()
            .map(|t| Ident::new(t.to_token_stream().to_string().as_str(), Span::call_site()))
            .or(bean_factory_info.ident_type.clone())
            .unwrap();

        let abstract_type: &Path = bean_factory_info.abstract_type.as_ref().unwrap();

        let profile_ident = Ident::new(Profile::default().profile.as_str(), Span::call_site());

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
                        let mut inner = #struct_type::default();
                        #(
                            let bean_def: BeanDefinition<#field_types>
                                = <ListableBeanFactory as BeanFactory<#field_types, #profile_ident>>::get_bean(listable_bean_factory);
                            let arc_bean_def: Arc<#field_types> = bean_def.inner;
                            inner.#field_idents = arc_bean_def.clone();
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>>
                                = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>, #profile_ident>>::get_bean(
                                    listable_bean_factory
                                );
                            let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                            inner.#mutable_identifiers = arc_bean_def.clone();
                        )*
                        #(
                            let arc_bean_def = <ListableBeanFactory as BeanFactory<#abstract_field_types, #profile_ident>>::get_bean(listable_bean_factory)
                                    .inner.clone();
                            inner.#abstract_field_idents = arc_bean_def;
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<dyn #abstract_mutable_field_types>>
                                = <ListableBeanFactory as MutableBeanFactory<Mutex<dyn #abstract_mutable_field_types>, #profile_ident>>::get_bean(
                                    listable_bean_factory
                                );
                            let arc_bean_def: Arc<Mutex<dyn #abstract_mutable_field_types>> = bean_def.inner;
                            inner.#abstract_mutable_idents = arc_bean_def.clone();
                        )*
                        let bean_def = BeanDefinition {
                            inner: Arc::new(inner)
                        };

                        bean_def
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

    fn new_bean_factory_generator(beans: &Vec<Bean>, abstract_beans: &Vec<(Bean, AutowireType, Profile)>) -> Self {
        let default_bean_factory = beans.iter()
            .map(|b| Self::create_concrete_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        let abstract_bean_factories = abstract_beans.iter()
            .map(|b| Self::create_abstract_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        Self {
            concrete_bean_factories: default_bean_factory,
            abstract_bean_factories,
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


pub struct PrototypeBeanFactoryGenerator {
    concrete_bean_factories: Vec<BeanFactoryInfo>,
    abstract_bean_factories: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for PrototypeBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

impl BeanFactoryGenerator for PrototypeBeanFactoryGenerator {
    fn create_concrete_bean_factories_for_bean(
        bean_factory_info: &BeanFactoryInfo
    ) -> TokenStream {
        let default_profile = Ident::new(Profile::default().profile.as_str(), Span::call_site());
        let injectable_code = quote! {
                //
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

    fn new_bean_factory_generator(beans: &Vec<Bean>, abstract_beans: &Vec<(Bean, AutowireType, Profile)>) -> Self {
        let bean_factories_to_implement = beans.iter()
            .map(|b| Self::create_concrete_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        let abstract_bean_factories = abstract_beans.iter()
            .map(|b| Self::create_abstract_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        Self {
            concrete_bean_factories: bean_factories_to_implement,
            abstract_bean_factories,
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
