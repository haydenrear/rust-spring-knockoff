use syn::{ItemStruct, parse2, Type};
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
use crate::module_macro_lib::bean_parser::{BeanDependencyParser};
use crate::module_macro_lib::item_parser::{get_profiles, ItemParser};
use crate::module_macro_lib::module_tree::Bean;
use crate::module_macro_lib::parse_container::ParseContainer;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use quote::{quote, ToTokens};

pub struct ItemStructParser;

impl ItemParser<ItemStruct> for ItemStructParser {
    fn parse_item(parse_container: &mut ParseContainer, item_struct: &mut ItemStruct, path_depth: Vec<String>) {
        log_message!("adding type with name {}", item_struct.ident.clone().to_token_stream().to_string());

        parse_container.initializer.field_augmenter.process(item_struct);

        parse_container.injectable_types_builder.get_mut(&item_struct.ident.to_string().clone())
            .map(|struct_impl: &mut Bean| {
                struct_impl.struct_found = Some(item_struct.clone());
                struct_impl.ident =  Some(item_struct.ident.clone());
                struct_impl.fields = vec![item_struct.fields.clone()];
                struct_impl.bean_type = BeanDependencyParser::get_bean_type_opt(&item_struct.attrs);
                struct_impl.id = item_struct.ident.clone().to_string();
            })
            .or_else(|| {
                let item_struct_ident = &item_struct.ident;
                let self_ty = quote! {
                    #item_struct_ident
                };
                let struct_type = parse2::<Type>(self_ty);
                let mut impl_found = Bean {
                    struct_type: struct_type.ok(),
                    struct_found: Some(item_struct.clone()),
                    traits_impl: vec![],
                    enum_found: None,
                    path_depth: path_depth.clone(),
                    attr: vec![],
                    deps_map: vec![],
                    id: item_struct.ident.clone().to_string(),
                    profile: get_profiles(&item_struct.attrs),
                    ident: Some(item_struct.ident.clone()),
                    fields: vec![item_struct.fields.clone()],
                    bean_type: BeanDependencyParser::get_bean_type_opt(&item_struct.attrs),
                    mutable: SynHelper::get_attr_from_vec(&item_struct.attrs, vec!["mutable_bean"])
                        .map(|_| true)
                        .or(Some(false))
                        .unwrap(),
                    aspect_info: vec![],
                };
                parse_container.injectable_types_builder.insert(item_struct.ident.to_string().clone(), impl_found);
                None
            });

    }
}
