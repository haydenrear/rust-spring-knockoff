use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

/**
This is the runtime application context.
 **/
pub trait ApplicationContext {
    fn get_bean_by_type_id<T,P>(&self, type_id: TypeId) -> Option<Arc<T>>
        where P: Profile, T: 'static + Send + Sync;
    fn get_bean_by_qualifier<T,P>(&self, qualifier: String) -> Option<Arc<T>>
        where P: Profile, T: 'static + Send + Sync;
    fn get_bean<T,P>(&self) -> Option<Arc<T>>
        where P: Profile, T: 'static + Send + Sync;
    fn get_beans(&self) -> Vec<Arc<dyn Any + Send + Sync>>;
    fn new() -> Self;
}

pub trait Profile {
    fn name() -> String;
}

pub trait AbstractListableFactory<P: Profile> {
    fn new() -> Self;
    fn get_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<T>>;
    fn get_mutable_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<Mutex<T>>>;
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
