use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

/**
This is the runtime application context.
 **/
pub trait ApplicationContext {
    fn new() -> Self;
    fn get_bean_for_profile<T: Any + Send + Sync, P: Profile>(&self) -> Option<Arc<T>>;
    fn get_bean<T: Any + Send + Sync>(&self) -> Option<Arc<T>>;
}

pub trait Profile {
    fn name() -> String;
}

pub trait AbstractListableFactory<P: Profile> {
    fn new() -> Self;
    fn get_beans(&self) -> Vec<Arc<dyn Any + Send + Sync>>;
    fn get_mutable_beans(&self) -> Vec<Arc<dyn Any + Send + Sync>>;
}

pub trait ContainsBeans {
    fn contains_bean_type(&self, type_id: &TypeId) -> bool;
    fn contains_mutable_bean_type(&self, type_id: &TypeId) -> bool;
    fn get_bean_types(&self) -> Vec<TypeId>;
    fn get_mutable_bean_types(&self) -> Vec<TypeId>;
    fn contains_type<T: 'static + Send + Sync>(&self) -> bool;
    fn contains_mutable_type<T: 'static + Send + Sync>(&self) -> bool;
}
