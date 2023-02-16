use std::any::TypeId;
use std::sync::Arc;

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
    fn get_beans<T>(&self) -> Vec<Arc<T>>
        where T: 'static + Send + Sync;
    fn new() -> Self;
}

pub trait Profile {
    fn name() -> String;
}

pub trait AbstractListableFactory<P: Profile> {
    fn new() -> Self;
    fn get_bean_definition<T: 'static + Send + Sync>(&self) -> Option<Arc<T>>;
    fn get_beans<T: 'static + Send + Sync>(&self) -> Vec<Arc<T>>;
}

pub trait ContainsBeans {
    fn contains_bean_type(&self, type_id: &TypeId) -> bool;
    fn get_bean_types(&self) -> Vec<TypeId>;
    fn contains_type<T: 'static + Send + Sync>(&self) -> bool;
}
