use syn::{ItemStruct, parse2, Type};
use codegen_utils::syn_helper::SynHelper;
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
use crate::module_macro_lib::bean_parser::{BeanDependencyParser};
use crate::module_macro_lib::item_parser::{get_all_generic_ty_bounds, get_profiles, ItemParser};
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::parse_container::ParseContainer;

use quote::{quote, ToTokens};
use crate::module_macro_lib::util::ParseUtil;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("item_struct_parser.rs");

use crate::FieldAugmenterImpl;

pub struct ItemStructParser;

impl ItemParser<ItemStruct> for ItemStructParser {
    fn parse_item(parse_container: &mut ParseContainer, item_struct: &mut ItemStruct, path_depth: Vec<String>) {
        // TODO: filter
        if !Self::is_bean(&item_struct.attrs) {
            return;
        }

        log_message!("adding type with name {}", item_struct.ident.clone().to_token_stream().to_string());

        let field_augmenter = FieldAugmenterImpl {};

        field_augmenter.process(item_struct);
        get_all_generic_ty_bounds(&item_struct.generics);

        parse_container.injectable_types_builder.get_mut(&item_struct.ident.to_string().clone())
            .map(|struct_impl: &mut BeanDefinition| {
                struct_impl.struct_found = Some(item_struct.clone());
                struct_impl.ident =  Some(item_struct.ident.clone());
                struct_impl.fields = vec![item_struct.fields.clone()];
                struct_impl.bean_type = BeanDependencyParser::get_bean_type_opt(&item_struct.attrs);
                struct_impl.id = item_struct.ident.clone().to_string();
                struct_impl.path_depth = path_depth.clone();
                struct_impl.declaration_generics = Some(item_struct.generics.clone())
            })
            .or_else(|| {
                let item_struct_ident = &item_struct.ident;
                let self_ty = quote! {
                    #item_struct_ident
                };
                let struct_type = parse2::<Type>(self_ty);
                let mut impl_found = BeanDefinition {
                    struct_type: struct_type.ok(),
                    struct_found: Some(item_struct.clone()),
                    traits_impl: vec![],
                    enum_found: None,
                    path_depth: path_depth.clone(),
                    deps_map: vec![],
                    id: item_struct.ident.clone().to_string(),
                    profile: get_profiles(&item_struct.attrs),
                    ident: Some(item_struct.ident.clone()),
                    fields: vec![item_struct.fields.clone()],
                    bean_type: BeanDependencyParser::get_bean_type_opt(&item_struct.attrs),
                    mutable: ParseUtil::does_attr_exist(&item_struct.attrs, &vec!["mutable_bean"]),
                    factory_fn: None,
                    declaration_generics: Some(item_struct.generics.clone()),
                };
                parse_container.injectable_types_builder.insert(item_struct.ident.to_string().clone(), impl_found);
                None
            });

    }
}
