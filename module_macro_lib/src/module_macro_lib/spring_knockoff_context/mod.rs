use std::any::TypeId;
use proc_macro2::TokenStream;
use syn::Type;

/**
This is the runtime application context.
**/
pub trait ApplicationContext {
    fn get_bean_by_type_id<T>(type_id: TypeId) -> T;
    fn get_bean_by_qualifier<T>(qualifier: String) -> T;
}

pub trait BeanFactory {
    fn get_bean<T>() -> T;
    fn get_bean_type_by_type_id<T>(type_id: TypeId) -> Type;
    fn get_bean_type_by_qualifier<T>(qualifier: String) -> Type;
    fn contains_bean<T>() -> bool;
}

pub trait FactoryBean<T> {
    fn get_bean() -> T;
    fn get_bean_type_id() -> TypeId;
    fn get_bean_type() -> Type;
    fn is_singleton() -> bool;
}

