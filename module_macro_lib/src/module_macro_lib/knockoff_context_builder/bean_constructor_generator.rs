use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};
use syn::Type;
use module_macro_shared::bean::Bean;
use crate::module_macro_lib::knockoff_context_builder::bean_factory_info::BeanFactoryInfo;
use crate::module_macro_lib::knockoff_context_builder::token_stream_generator::TokenStreamGenerator;
use module_macro_shared::bean::BeanDefinitionType;
use module_macro_shared::profile_tree::ProfileTree;

pub struct BeanConstructorGenerator {
    beans_to_create_constructor_for: Vec<BeanFactoryInfo>
}

impl BeanConstructorGenerator {

    pub(crate) fn create_bean_constructor_generator(profile_tree: Vec<BeanFactoryInfo>) -> Box<dyn TokenStreamGenerator> {
        Box::new(Self {
            beans_to_create_constructor_for: profile_tree
        })
    }

    pub(crate) fn create_constructor(bean_factory_info: &BeanFactoryInfo) -> TokenStream {
        let (field_types, field_idents, concrete_field,
            mutable_identifiers, mutable_field_types, concrete_mutable_type,
            abstract_field_idents, abstract_field_types, concrete_abstract_types,
            abstract_mutable_identifiers, abstract_mutable_field_types, concrete_mutable_abstract)
            = bean_factory_info.get_field_types();
        let struct_type = bean_factory_info.concrete_type.as_ref().unwrap();
        let default_type = bean_factory_info.default_field_info.iter().map(|f| &f.field_type)
            .collect::<Vec<&Type>>();
        let default_ident = bean_factory_info.default_field_info.iter().map(|f| &f.field_ident)
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

impl TokenStreamGenerator for BeanConstructorGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        let mut ts = TokenStream::default();
        self.beans_to_create_constructor_for.iter().for_each(|b| {
            ts.append_all(Self::create_constructor(b))
        });
        ts
    }
}