use proc_macro::TokenStream;
use std::fs::File;
use std::path::Path;
use quote::quote;
use syn::{Item, parse_macro_input, Type};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::*;
use lazy_static::lazy_static;
use knockoff_helper::project_directory;
use std::sync::Mutex;
use proc_macro2::Ident;

import_logger_root!("derive_lib.rs", concat!(project_directory!(), "/log_out/config_properties_macro.log"), "derive");


/// 1. Create an index by the value attribute for the different configuration properties - this is
///    added in a new provider. Also, create an index for each of the properties in the beans,
///    so you can access it by value.bean_property_name.
/// 2. Implement a new method below that passes in the string representation of the config property
///    bean and produces the Configuration property bean.
/// 3. Add the ConfigurationProperties beans as per normal to the container but have another
///    BeanFactory that uses the PropertiesParser and passes it in to create the beans, and then
///    include these bean types in the BeanFactoryInfo. This means overriding the construction logic
///    like is happening in factory fn bean factory.
/// 4. Also, add the injection of the properties that have #[property(value.property_name)].
///    For each of the field dependencies, add this property type, and then add it to bean
///    factory info as a type of bean. This means adding to current initialization logic. It
///    will require that the type of the config property will be retrieved using the bean factory
///    and then the field on that config property will be cloned and added.
#[proc_macro_derive(ConfigurationProperties, attributes(value))]
pub fn configuration_properties(ts: TokenStream) -> TokenStream {
    let mut item_found: Item = parse_macro_input!(ts as Item).into();
    if let Item::Struct(item_struct) = item_found {
        let attr = SynHelper::get_attr_from_vec_prop(&item_struct.attrs, &vec!["value"]);
        info!("Found configuration properties with attr {:?}.", &attr);
        let field_names = item_struct.fields.iter()
            .flat_map(|f| {
                f.ident.iter()
            })
            .collect::<Vec<&Ident>>();
        let field_tys = item_struct.fields.iter()
            .map(|f| {
                &f.ty
            })
            .collect::<Vec<&Type>>();
        let item_struct_ident = &item_struct.ident;
        if field_names.len() != field_tys.len() {
            error!("Field names did not match field types.");
        } else {
            quote! {
                impl #item_struct_ident {
                    pub fn new(profile: String) -> Self {

                    }
                }
                #(
                    #field_names
                )*
            };
        }
    }
    TokenStream::default()
}