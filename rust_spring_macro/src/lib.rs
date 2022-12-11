pub mod module_post_processor {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use syn::{FieldsNamed, FieldsUnnamed, ItemMod, ItemStruct};
    use syn::parse::Parse;

    pub trait ModulePostProcessor {
        fn process_modules(&self, module_item: &mut ItemMod);
    }

    pub trait ModuleFieldPostProcessor {
        fn process_fields(&self, field_named: &mut FieldsNamed);
        fn process_unnamed_fields(&self, fields_unnamed: &mut FieldsUnnamed);
    }

    pub trait ModuleStructPostProcessor: Parse {
        fn process_struct(&self, struct_item: &mut ItemStruct);
    }

    pub struct Container {
        prototype: HashMap<TypeId, Box<dyn PrototypeFactory<Box<dyn Any>>>>,
        singleton: HashMap<TypeId, Box<dyn SingletonFactory<Box<dyn Any>>>>
    }

    pub trait PrototypeFactory<T> : Send + Sync {
        fn create(&self) -> T;
    }

    pub trait SingletonFactory<T> : Send + Sync {
        fn create(&self) -> T;
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
