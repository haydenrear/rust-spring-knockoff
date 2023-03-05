use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::token::Mut;
use syn::Type;
use codegen_utils::syn_helper::SynHelper;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use crate::module_macro_lib::module_tree::{Bean, BeanPath};
use knockoff_logging::{initialize_log, use_logging};

use_logging!();
initialize_log!();

use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;


#[derive(Clone, Default)]
pub struct BeanFactoryInfo {
    field_types: Vec<Type>,
    field_idents: Vec<Ident>,
    mutable_field_types: Vec<Type>,
    mutable_field_idents: Vec<Ident>,
    struct_type: Option<Type>,
    ident_type: Option<Ident>
}

pub trait BeanFactoryGenerator: TokenStreamGenerator {

    fn create_concrete_bean_factories<T: ToTokens>(
        field_types: &Vec<Type>,
        field_idents: &Vec<Ident>,
        mutable_field_types: &Vec<Type>,
        mutable_field_idents: &Vec<Ident>,
        struct_type: &T,
    ) -> TokenStream;

    fn create_abstract_bean_factories<T: ToTokens>(
        field_types: &Vec<Type>,
        field_idents: &Vec<Ident>,
        struct_type: &T,
    ) -> TokenStream;

    fn new_bean_factory_generators(beans: &Vec<Bean>) -> Vec<Box<dyn TokenStreamGenerator>> {
        vec![
            Box::new(MutableBeanFactoryGenerator::new_bean_factory_generator(beans)) as Box<dyn TokenStreamGenerator>,
            Box::new(FactoryBeanBeanFactoryGenerator::new_bean_factory_generator(beans)) as Box<dyn TokenStreamGenerator>,
            Box::new(PrototypeBeanFactoryGenerator::new_bean_factory_generator(beans)) as Box<dyn TokenStreamGenerator>,
        ]
    }

    fn create_bean_factory_info(bean: &Bean) -> BeanFactoryInfo {
        let mutable_fields = Self::get_mutable_singleton_field_ids(bean);
        let fields = Self::get_singleton_field_ids(bean);
        BeanFactoryInfo {
            field_types: fields.0,
            field_idents: fields.1,
            mutable_field_types: mutable_fields.0,
            mutable_field_idents: mutable_fields.1,
            struct_type: bean.struct_type.clone(),
            ident_type: bean.ident.clone(),
        }
    }

    fn get_singleton_field_ids(bean: &Bean) -> (Vec<Type>, Vec<Ident>) {
        let non_mutable = Self::get_field_ids(bean, &|b| {
            b.path_segments.iter().all(|path_part| !path_part.is_mutable())
        });
        non_mutable.0.iter().for_each(|non_mutable_type| {
            log_message!("{} is non mutable field type.", SynHelper::get_str(non_mutable_type.clone()));
        });
        non_mutable
    }

    fn get_field_ids(token_type: &Bean, matcher: &dyn Fn(&BeanPath) -> bool) -> (Vec<Type>, Vec<Ident>) {

        let field_types = token_type.deps_map
            .clone().iter()
            .map(|d| {
                log_message!("Parsing field id dep type {}.", SynHelper::get_str(d.bean_info.type_of_field.clone()).as_str());
                log_message!("Parsing field id dep type {}.", SynHelper::get_str(d.bean_info.field.clone()).as_str());
                d.clone()
                    .bean_type_path
                    .filter(|btp| {
                        let f = matcher(btp);
                        f
                    })
                    .map(|type_path| {
                        type_path.get_autowirable_type()
                    })
                    .flatten()
                    .map(|a_type| {
                        log_message!("Parsed dep type {} using bean type path.", SynHelper::get_str(a_type.clone()));
                        a_type
                    })
                    .or_else(|| {
                        log_message!("Could not parse dep type with previous id.");
                        None
                    })
            })
            .flat_map(|f| f.map(|t| vec![t]).or(Some(vec![])))
            .flat_map(|f| f)
            .collect::<Vec<Type>>();

        let identifiers = token_type.deps_map
            .clone().iter()
            .flat_map(|d| {
                match &d.bean_info.field.ident {
                    None => {
                        vec![]
                    }
                    Some(identifier) => {
                        vec![identifier.clone()]
                    }
                }
            })
            .collect::<Vec<Ident>>();

        (field_types, identifiers)
    }

    fn get_mutable_singleton_field_ids(token_type: &Bean) -> (Vec<Type>, Vec<Ident>) {
        let non_mutable = Self::get_field_ids(token_type, &|b| {
            b.path_segments.iter().any(|path_part| path_part.is_mutable())
        });
        non_mutable.0.iter().for_each(|non_mutable_type| {
            log_message!("{} is mutable field type.", SynHelper::get_str(non_mutable_type.clone()));
        });
        non_mutable
    }


    fn generate_factories(&self) -> TokenStream {
        let mut ts = TokenStream::default();
        self.get_factories().iter()
            .for_each(|b| {
                if b.ident_type.is_some() {
                    ts.append_all(Self::create_concrete_bean_factories(
                        &b.field_types, &b.field_idents, &b.mutable_field_types,
                        &b.mutable_field_idents, &b.ident_type.clone().unwrap())
                    );
                } else if b.struct_type.is_some() {
                    ts.append_all(Self::create_concrete_bean_factories
                        (&b.field_types, &b.field_idents, &b.mutable_field_types,
                         &b.mutable_field_idents, &b.struct_type.clone().unwrap()));
                }
            });
        ts
    }


    fn new_bean_factory_generator(beans: &Vec<Bean>) -> Self;

    fn get_factories(&self) -> Vec<BeanFactoryInfo>;
}

pub struct MutableBeanFactoryGenerator {
    bean_factories_to_implement: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for MutableBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

impl BeanFactoryGenerator for MutableBeanFactoryGenerator {
    fn create_concrete_bean_factories<T: ToTokens>(
        field_types: &Vec<Type>,
        field_idents: &Vec<Ident>,
        mutable_field_types: &Vec<Type>,
        mutable_identifiers: &Vec<Ident>,
        struct_type: &T,
    ) -> TokenStream {
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
                            let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>>
                                = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>>>::get_bean(
                                    listable_bean_factory
                                );
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



    fn create_abstract_bean_factories<T: ToTokens>(field_types: &Vec<Type>, field_idents: &Vec<Ident>, struct_type: &T) -> TokenStream {
        TokenStream::default()
    }

    fn new_bean_factory_generator(beans: &Vec<Bean>) -> Self {
        let bean_factories_to_implement = beans.iter()
            .map(|b| Self::create_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        Self {
            bean_factories_to_implement
        }
    }

    fn get_factories(&self) -> Vec<BeanFactoryInfo> {
        self.bean_factories_to_implement.clone()
    }
}

pub struct FactoryBeanBeanFactoryGenerator {
    bean_factories_to_implement: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for FactoryBeanBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

impl BeanFactoryGenerator for FactoryBeanBeanFactoryGenerator {
    fn create_concrete_bean_factories<T: ToTokens>(
        field_types: &Vec<Type>,
        field_idents: &Vec<Ident>,
        mutable_field_types: &Vec<Type>,
        mutable_identifiers: &Vec<Ident>,
        struct_type: &T,
    ) -> TokenStream {
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

    fn create_abstract_bean_factories<T: ToTokens>(field_types: &Vec<Type>, field_idents: &Vec<Ident>, struct_type: &T) -> TokenStream {
        TokenStream::default()
    }

    fn new_bean_factory_generator(beans: &Vec<Bean>) -> Self {
        let bean_factories_to_implement = beans.iter()
            .map(|b| Self::create_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        Self {
            bean_factories_to_implement
        }
    }

    fn get_factories(&self) -> Vec<BeanFactoryInfo> {
        self.bean_factories_to_implement.clone()
    }
}


pub struct PrototypeBeanFactoryGenerator {
    bean_factories_to_implement: Vec<BeanFactoryInfo>
}

impl TokenStreamGenerator for PrototypeBeanFactoryGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_factories()
    }
}

impl BeanFactoryGenerator for PrototypeBeanFactoryGenerator {
    fn create_concrete_bean_factories<T: ToTokens>(
        field_types: &Vec<Type>,
        field_idents: &Vec<Ident>,
        mutable_field_types: &Vec<Type>,
        mutable_identifiers: &Vec<Ident>,
        struct_type: &T,
    ) -> TokenStream {
        log_message!("Creating bean factory with the following mutable field types: ");
        mutable_field_types.iter().for_each(|m| {
            log_message!("{} is the mutable field type.", SynHelper::get_str(m.clone()));
        });
        log_message!("Creating bean factory with the following field types: ");
        field_types.iter().for_each(|m| {
            log_message!("{} is the mutable field type.", SynHelper::get_str(m.clone()));
        });
        let injectable_code = quote! {

                impl PrototypeFactoryBean<#struct_type> for ListableBeanFactory {

                    fn get_prototype_bean(&self) -> PrototypeBeanDefinition<#struct_type> {
                        let mut inner = #struct_type::default();
                        #(
                            let bean_def: BeanDefinition<#field_types> = <ListableBeanFactory as BeanFactory<#field_types>>::get_bean(&self);
                            let arc_bean_def: Arc<#field_types> = bean_def.inner;
                            inner.#field_idents = arc_bean_def.clone();
                        )*
                        #(
                            let bean_def: MutableBeanDefinition<Mutex<#mutable_field_types>> = <ListableBeanFactory as MutableBeanFactory<Mutex<#mutable_field_types>>>::get_bean(&self);
                            let arc_bean_def: Arc<Mutex<#mutable_field_types>> = bean_def.inner;
                            inner.#mutable_identifiers = arc_bean_def.clone();
                        )*
                        PrototypeBeanDefinition {
                            inner: Arc::new(inner)
                        }
                    }

                    fn get_bean_type_id() -> TypeId {
                        TypeId::of::<#struct_type>().clone()
                    }

                }

        };

        injectable_code.into()
    }

    fn create_abstract_bean_factories<T: ToTokens>(field_types: &Vec<Type>, field_idents: &Vec<Ident>, struct_type: &T) -> TokenStream {
        TokenStream::default()
    }

    fn new_bean_factory_generator(beans: &Vec<Bean>) -> Self {
        let bean_factories_to_implement = beans.iter()
            .map(|b| Self::create_bean_factory_info(b))
            .collect::<Vec<BeanFactoryInfo>>();
        Self {
            bean_factories_to_implement
        }
    }

    fn get_factories(&self) -> Vec<BeanFactoryInfo> {
        self.bean_factories_to_implement.clone()
    }
}
