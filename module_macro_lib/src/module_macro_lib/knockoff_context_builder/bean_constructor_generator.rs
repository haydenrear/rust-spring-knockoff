use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt, ToTokens};
use syn::{parse2, Type};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::error;
use module_macro_shared::bean::BeanDefinition;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use module_macro_shared::bean::BeanDefinitionType;
use module_macro_shared::profile_tree::ProfileTree;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("bean_constructor_generator.rs");

pub struct BeanConstructorGenerator {
    beans_to_create_constructor_for: Vec<BeanFactoryInfo>
}

impl BeanConstructorGenerator {

    pub(crate) fn create_bean_constructor_generator(profile_tree: Vec<BeanFactoryInfo>) -> Box<dyn TokenStreamGenerator> {
        Box::new(Self {
            beans_to_create_constructor_for: profile_tree.into_iter()
                .filter(|b| b.constructable)
                .collect::<Vec<_>>()
        })
    }

    pub(crate) fn create_constructor(bean_factory_info: &BeanFactoryInfo) -> TokenStream {
        let struct_type = bean_factory_info.concrete_type.clone().or_else(||
            bean_factory_info.ident_type.clone()
                .map(|i| parse2::<Type>(i.to_token_stream()).ok())
                .flatten()
        ).unwrap();
        if bean_factory_info.is_enum
            && bean_factory_info.is_default
            && bean_factory_info.constructable {
            info!("Enum provided with default. Delegating to default for enum.");
            let constructor = quote! {
                impl #struct_type {
                    fn new() -> Self {
                        Self::default()
                    }
                }
            };
            constructor.into()
        } else if bean_factory_info.is_enum {
            panic!("Attempted to create constructor for enum type. Not currently supported.");
        } else {
            let (field_types, field_idents, concrete_field,
                mutable_identifiers, mutable_field_types, concrete_mutable_type,
                abstract_field_idents, abstract_field_types, concrete_abstract_types,
                abstract_mutable_identifiers, abstract_mutable_field_types, concrete_mutable_abstract)
                = bean_factory_info.get_field_types();
            info!("Creating constructor for {:?}", SynHelper::get_str(&struct_type));
            let default_type = bean_factory_info.default_field_info
                .iter()
                .map(|f| &f.field_type)
                .collect::<Vec<&Type>>();
            let default_ident = bean_factory_info.default_field_info
                .iter()
                .map(|f| &f.field_ident)
                .collect::<Vec<&Ident>>();
            let constructor = quote! {
            impl #struct_type {
                fn new(
                    #(#field_idents: Arc<#field_types>,)*
                    #(#mutable_identifiers: Arc<Mutex<#mutable_field_types>>,)*
                    #(#abstract_field_idents: Arc<#abstract_field_types>,)*
                    #(#abstract_mutable_identifiers: Arc<Mutex<Box<#abstract_mutable_field_types>>>,)*
                ) -> Self {
                    Self {
                        #(#default_ident: #default_type::default(),)*
                        #(#field_idents,)*
                        #(#mutable_identifiers,)*
                        #(#abstract_field_idents,)*
                        #(#abstract_mutable_identifiers,)*
                    }
                }
            }
        };
            constructor.into()
        }
    }
}

impl TokenStreamGenerator for BeanConstructorGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        let mut ts = TokenStream::default();
        self.beans_to_create_constructor_for.iter()
            .filter(|bean_factory_info| {
                if bean_factory_info.concrete_type.as_ref().is_none() && bean_factory_info.ident_type.as_ref().is_none() {
                    error!(
                        "Failed to create factory for {:?}, as the concrete type was none. \
                        In the future, try using the ident type.",
                        SynHelper::get_str(bean_factory_info.ident_type.as_ref().unwrap()));
                    false
                } else if bean_factory_info.constructable {
                    true
                } else {
                    false
                }
            })
            .for_each(|b| {
                ts.append_all(Self::create_constructor(b))
            });
        ts
    }
}