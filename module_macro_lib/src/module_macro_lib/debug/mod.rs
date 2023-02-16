use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ptr::write;
use proc_macro2::Ident;
use syn::{ItemFn, Type};
use quote::ToTokens;
use crate::module_macro_lib::module_tree::{BeanDefinition, FunctionType, Trait};
use crate::module_macro_lib::profile_tree::ProfileTree;

impl Debug for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionType::Singleton(item_fn, qual, f_type) => {
                FunctionType::debug_key_value("Singleton", "FunctionType: ", f);
                Self::debug_ftype(item_fn, qual, f_type, f);
            }
            FunctionType::Prototype(item_fn, qual, f_type) => {
                FunctionType::debug_key_value("Prototype", "FunctionType: ", f);
                Self::debug_ftype(item_fn, qual, f_type, f);
            }
        };
        Ok(())
    }
}

impl FunctionType {

    fn debug_ftype(item_fn: &ItemFn, qual: &Option<String>, f_type: &Option<Type>, f: &mut Formatter) {
        Self::debug_key_value(item_fn.to_token_stream().to_string().as_str(), "item_fn: ", f);
        qual.clone().map(|qualifier| {
            Self::debug_key_value(qualifier.as_str(), "qualifier: ", f);
        });
        f_type.clone().map(|f_type| {
            Self::debug_key_value(f_type.to_token_stream().to_string().as_str(), "type: ", f);
        });
    }

    fn debug_key_value(value: &str, key: &str, formatter: &mut fmt::Formatter) {
        write!(formatter, "{}", key).expect("Could not Debug");
        writeln!(formatter, "{}", value).expect("Could not Debug");
    }

}

impl Debug for BeanDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("bean_type");
        self.qualifier.as_ref().and_then(|q| {
            debug_struct.field("qualifier", &q.as_str());
            None::<String>
        });
        self.bean_type_ident.as_ref().and_then(|q| {
            debug_struct.field("bean_type_ident", &q.to_token_stream().to_string().as_str());
            None::<Ident>
        });
        self.bean_type_type.as_ref().and_then(|q| {
            debug_struct.field("bean_type_type", &q.to_token_stream().to_string().as_str());
            None::<Type>
        });
        debug_struct.finish()
    }
}

impl Debug for ProfileTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_map = f.debug_map();
        self.injectable_types.iter()
            .for_each(|p| {
                debug_map.entry(&"profile", &p.0.clone() as &dyn Debug);
            });
        debug_map.finish()
    }
}
