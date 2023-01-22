use std::any::TypeId;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::__private::str;
use syn::Type;


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

    pub fn create_bean_factory() -> TokenStream {
        let ts = quote! {

            pub struct BeanDefinition<T> {
                inner: Option<T>
            }

            pub trait BeanFactory<T> {
                fn get_bean(&self) -> BeanDefinition<T>;
            }

            // pub trait ContainsBean<T, U> where T: BeanDefinition<U> {
            //     fn contains_bean_type(type_id: TypeId) -> bool;
            //     fn get_bean_types() -> Vec<TypeId>;
            // }

            pub trait FactoryBean<T> {
                fn get_bean() -> BeanDefinition<T>;
                // fn get_bean_type_id() -> TypeId;
                // fn get_bean_type() -> Type;
                // fn is_singleton() -> bool;
            }

            pub struct ListableBeanFactory {
            }
        };
        ts.into()
    }

    pub fn gen_autowire_code_ident(field_types: Vec<Type>, field_idents: Vec<Ident>, struct_type: Ident)
                             -> TokenStream
    {
        let injectable_code = quote! {

                impl BeanFactory<#struct_type> for ListableBeanFactory {
                    fn get_bean(&self) -> BeanDefinition<#struct_type> {
                        let this_component = <BeanDefinition<#struct_type>>::get_bean();
                        this_component
                    }
                }

                impl FactoryBean<#struct_type> for BeanDefinition<#struct_type> {
                    fn get_bean() -> Self {
                        let mut inner = #struct_type::default();
                        #(
                            inner.#field_idents = ListableBeanFactory::get_bean::<#field_types>();
                        )*
                        Self {
                            inner: Some(inner)
                        }
                    }

                }
        };
        injectable_code.into()
    }

    pub fn gen_autowire_code(field_types: Vec<Type>, field_idents: Vec<Ident>, struct_type: Type)
                             -> TokenStream
    {
        let injectable_code = quote! {

                impl BeanFactory<#struct_type> for ListableBeanFactory {
                    fn get_bean(&self) -> BeanDefinition<#struct_type> {
                        let this_component = <BeanDefinition<#struct_type>>::get_bean();
                        this_component
                    }
                }

                impl FactoryBean<#struct_type> for BeanDefinition<#struct_type> {
                    fn get_bean() -> Self {
                        let mut inner = #struct_type::default();
                        #(
                            inner.#field_idents = ListableBeanFactory::get_bean::<#field_types>();
                        )*
                        Self {
                            inner: Some(inner)
                        }
                    }

                }
        };
        injectable_code.into()
    }
}


