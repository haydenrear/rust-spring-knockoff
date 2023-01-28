use std::any::{Any, TypeId};
use std::collections::HashMap;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::__private::str;
use syn::Type;
use crate::module_macro_lib::module_tree::DepImpl;


pub struct ApplicationContextGenerator {
}

impl ApplicationContextGenerator {

    pub fn create_application_context() -> TokenStream {
        let ts = quote! {
            /**
            This is the runtime application context.
             **/
            pub trait ApplicationContext {
                fn get_bean_by_type_id<T>(type_id: TypeId) -> T;
                fn get_bean_by_qualifier<T>(qualifier: String) -> T;
            }
        };
        ts.into()
    }

    // pub fn finish_bean_factory(bean_factory_types: Vec<Type>) -> TokenStream {
    //     let ts = quote! {
    //         impl <T> ContainsBean for BeanFactory<T> {
    //             fn contains_bean_type(type_id: TypeId) -> bool {
    //
    //             }
    //             fn get_bean_types() -> Vec<TypeId> {
    //                 let mut types = vec![];
    //                 #(
    //                     types.push()
    //                 )*
    //             }
    //         }
    //     };
    //
    //     ts.into()
    // }

    pub fn create_bean_factory() -> TokenStream {
        let ts = quote! {

            #[derive(Debug)]
            pub struct BeanDefinition<T: ?Sized> {
                inner: Arc<T>
            }

            impl <T: 'static + Send + Sync> BeanDefinition<T> {
                fn to_any(&self) -> BeanDefinition<dyn Any + Send + Sync> {
                    let inner: Arc<dyn Any + Send + Sync> = self.inner.clone() as Arc<dyn Any + Send + Sync>;
                    BeanDefinition {
                        inner: inner
                    }
                }
                fn get_bean_type_id(&self) -> TypeId {
                    self.inner.deref().type_id().clone()
                }
            }

            pub trait BeanFactory<T> {
                fn get_bean(&self) -> BeanDefinition<T>;
            }

            pub trait ContainsBeans {
                fn contains_bean_type(&self, type_id: &TypeId) -> bool;
                fn get_bean_types(&self) -> Vec<TypeId>;
                fn contains_type<T: 'static + Send + Sync>(&self) -> bool;
            }

            pub trait FactoryBean<T: 'static + Send + Sync> {
                fn get_bean(listable_bean_factory: &ListableBeanFactory) -> BeanDefinition<T>;
                fn get_bean_type_id(&self) -> TypeId;
                fn is_singleton() -> bool;
            }

            // pub trait AutowireCapableBeanFactory: ContainsBeans {
            // }

            #[derive(Default)]
            pub struct ListableBeanFactory {
                singleton_bean_definitions: HashMap<TypeId, BeanDefinition<dyn Any + Send + Sync>>
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

                fn get_bean_types(&self) -> Vec<TypeId> {
                    self.singleton_bean_definitions.keys()
                        .map(|type_id| type_id.clone())
                        .collect::<Vec<TypeId>>()
                }

                fn contains_type<T: 'static + Send + Sync>(&self) -> bool {
                    let type_id_to_search = TypeId::of::<T>();
                    self.singleton_bean_definitions.keys()
                        .any(|t| t.clone() == type_id_to_search.clone())
                }
            }

            pub trait BeanProvider<T> where T: 'static + Send + Sync {
                fn get_bean_singleton_ref(&self) -> Arc<T>;
                // fn create_prototype_bean(&self) -> Option<&'a BeanDefinition<T>>
            }
        };
        ts.into()
    }

    pub fn new_listable_bean_factory(beans_to_provide: Vec<&DepImpl>) -> TokenStream {
        let mut identifiers = vec![];
        let mut types_identifiers = vec![];
        for bean in beans_to_provide.iter() {
            if bean.ident.is_some() {
                println!("Implementing listable bean factory. Including: {}.", bean.ident.to_token_stream().to_string().clone());
                identifiers.push(bean.ident.clone().unwrap());
            } else if bean.struct_type.is_some() {
                types_identifiers.push(bean.struct_type.clone().unwrap());
            }
        }
        let new_listable_bean_factory = quote! {

            impl ListableBeanFactory {

                fn add_bean_definition<T: 'static + Send + Sync>(&mut self, bean_defin: BeanDefinition<T>) {
                    self.singleton_bean_definitions.insert(
                        bean_defin.get_bean_type_id().clone(),
                        bean_defin.to_any()
                    );
                }

                fn new() -> Self {
                    let mut singleton_bean_definitions: HashMap<TypeId, BeanDefinition<dyn Any + Send + Sync>> = HashMap::new();
                    let mut listable_bean_factory = ListableBeanFactory {
                        singleton_bean_definitions
                    };
                    #(
                        let next_bean_definition = <BeanDefinition<#identifiers>>::get_bean(&listable_bean_factory);
                        println!("Adding next bean definition {:?}.", next_bean_definition);
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

            }
        };
        new_listable_bean_factory.into()
    }

    pub fn gen_autowire_code_ident(field_types: Vec<Type>, field_idents: Vec<Ident>, struct_type: Ident)
                             -> TokenStream
    {
        let injectable_code = quote! {

                // impl BeanProvider<#struct_type> for ListableBeanFactory {
                //     fn get_bean_singleton_ref(&self) -> Option<BeanDefinition<#struct_type>> {
                        // let type_id = <BeanDefinition<#struct_type>>::get_bean_type_id();
                        // if self.contains_bean_type(type_id.clone()) && <BeanDefinition<#struct_type>>::is_singleton() {
                        //     let bean_definition = &self.bean_definitions(type_id.clone());
                        //     bean_definition
                        // }
                        // None
                    // }

                    // fn create_prototype_bean(&self) -> Option<&'a BeanDefinition<#struct_type>> {
                    // }
                // }

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
                            inner.#field_idents = ListableBeanFactory::<#field_types>::get_bean(listable_bean_factory);
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

    // pub fn gen_autowire_code(field_types: Vec<Type>, field_idents: Vec<Ident>, struct_type: Type)
    //                          -> TokenStream
    // {
    //     let injectable_code = quote! {
    //
    //             impl BeanFactory<#struct_type> for ListableBeanFactory {
    //                 fn get_bean(&self) -> BeanDefinition<#struct_type> {
    //                     let this_component = <BeanDefinition<#struct_type>>::get_bean();
    //                     this_component
    //                 }
    //             }
    //
    //             impl FactoryBean<#struct_type> for BeanDefinition<#struct_type> {
    //                 fn get_bean() -> Self {
    //                     let mut inner = #struct_type::default();
    //                     #(
    //                         inner.#field_idents = ListableBeanFactory::get_bean::<#field_types>();
    //                     )*
    //                     Self {
    //                         inner: Some(inner)
    //                     }
    //                 }
    //
    //                 fn get_bean_type_id(&self) -> Option<TypeId> {
    //                     self.inner.as_ref()
    //                         .and_then(|bean_type| Some(bean_type.type_id().clone()))
    //                 }
    //             }
    //     };
    //     injectable_code.into()
    // }
}


