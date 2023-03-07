use std::any::Any;
use std::ops::Deref;
use quote::ToTokens;
use syn::{Fields, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait};
use codegen_utils::syn_helper::SynHelper;
use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;
use crate::module_macro_lib::bean_parser::BeanParser;
use crate::module_macro_lib::fn_parser::FnParser;
use crate::module_macro_lib::parse_container::ParseContainer;

use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::module_parser::parse_item;
use crate::module_macro_lib::module_tree::{AutowireType, Bean, ModulesFunctions, Trait};


pub trait ItemParser<T: ToTokens> {
    fn parse_item(parse_container: &mut ParseContainer, item: &mut T, path_depth: Vec<String>);
}

pub struct ItemImplParser;

impl ItemImplParser{
    fn add_path(path_depth: &mut Vec<String>, impl_found: &ItemImpl) {
        let mut trait_impl = vec![];
        impl_found.trait_.clone().map(|trait_found| {
            trait_impl.push(trait_found.1.to_token_stream().to_string());
        });
        trait_impl.push(impl_found.self_ty.to_token_stream().to_string().clone());
        path_depth.push(trait_impl.join("|"));
    }

}

impl ItemParser<ItemImpl> for ItemImplParser {
    fn parse_item(parse_container: &mut ParseContainer, item_impl: &mut ItemImpl, mut path_depth: Vec<String>) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        log_message!("Doing create update impl.");

        Self::add_path(&mut path_depth, &item_impl);

        &mut parse_container.injectable_types_builder.get_mut(&id)
            .map(|bean: &mut Bean| {
                bean.traits_impl.push(
                    AutowireType {
                        item_impl: item_impl.clone(),
                        profile: vec![],
                        path_depth: path_depth.clone()
                    }
                );
            })
            .or_else(|| {
                let mut impl_found = Bean {
                    struct_type: Some(item_impl.self_ty.deref().clone()),
                    struct_found: None,
                    traits_impl: vec![
                        AutowireType {
                            item_impl: item_impl.clone(),
                            profile: vec![],
                            path_depth: path_depth.clone()
                        }
                    ],
                    enum_found: None,
                    attr: vec![],
                    deps_map: vec![],
                    id: id.clone(),
                    path_depth: vec![],
                    profile: vec![],
                    ident: None,
                    fields: vec![],
                    bean_type: None,
                    mutable: SynHelper::get_attr_from_vec(&item_impl.attrs, vec!["mutable_bean"])
                        .map(|_| true)
                        .or(Some(false)).unwrap(),
                    aspect_info: None,
                };
                parse_container.injectable_types_builder.insert(id.clone(), impl_found);
                None
            });

        log_message!("Adding method advice aspect now.");

        parse_container.add_method_advice_aspect(item_impl, &mut path_depth, &id);
    }
}

pub struct ItemStructParser;

impl ItemParser<ItemStruct> for ItemStructParser {
    fn parse_item(parse_container: &mut ParseContainer, item_struct: &mut ItemStruct, path_depth: Vec<String>) {
        log_message!("adding type with name {}", item_struct.ident.clone().to_token_stream().to_string());
        log_message!("adding type with name {}", item_struct.to_token_stream().to_string().clone());

        parse_container.initializer.field_augmenter.process(item_struct);

        parse_container.injectable_types_builder.get_mut(&item_struct.ident.to_string().clone())
            .map(|struct_impl: &mut Bean| {
                struct_impl.struct_found = Some(item_struct.clone());
                struct_impl.ident =  Some(item_struct.ident.clone());
                struct_impl.fields = vec![item_struct.fields.clone()];
                struct_impl.bean_type = BeanParser::get_bean_type(&item_struct.attrs, None, Some(item_struct.ident.clone()));
                struct_impl.id = item_struct.ident.clone().to_string();
            })
            .or_else(|| {
                let mut impl_found = Bean {
                    struct_type: None,
                    struct_found: Some(item_struct.clone()),
                    traits_impl: vec![],
                    enum_found: None,
                    path_depth: path_depth.clone(),
                    attr: vec![],
                    deps_map: vec![],
                    id: item_struct.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(item_struct.ident.clone()),
                    fields: vec![item_struct.fields.clone()],
                    bean_type: BeanParser::get_bean_type(&item_struct.attrs, None, Some(item_struct.ident.clone())),
                    mutable: SynHelper::get_attr_from_vec(&item_struct.attrs, vec!["mutable_bean"])
                        .map(|_| true)
                        .or(Some(false)).unwrap(),
                    aspect_info: None,
                };
                parse_container.injectable_types_builder.insert(item_struct.ident.to_string().clone(), impl_found);
                None
            });

    }
}

pub struct ItemModParser;

impl ItemParser<ItemMod> for ItemModParser {
    fn parse_item(parse_container: &mut ParseContainer, item_found: &mut ItemMod, mut path_depth: Vec<String>) {
        item_found.content.iter_mut()
            .flat_map(|mut c| c.1.iter_mut())
            .for_each(|i: &mut Item| parse_item(i, parse_container, &mut path_depth.clone()));
    }
}

pub struct ItemFnParser;

impl ItemParser<ItemFn> for ItemFnParser {
    fn parse_item(parse_container: &mut ParseContainer, item_fn: &mut ItemFn, path_depth: Vec<String>) {
        FnParser::to_fn_type(item_fn.clone())
            .map(|fn_found| {
                parse_container.fns.insert(item_fn.clone().type_id().clone(), ModulesFunctions{ fn_found: fn_found.clone() });
            })
            .or_else(|| {
                log_message!("Could not set fn type for fn named: {}", SynHelper::get_str(item_fn.sig.ident.clone()).as_str());
                None
            });
    }
}

pub struct ItemEnumParser;

impl ItemParser<ItemEnum> for ItemEnumParser {
    fn parse_item(parse_container: &mut ParseContainer, enum_to_add: &mut ItemEnum, path_depth: Vec<String>) {
        log_message!("adding type with name {}", enum_to_add.ident.clone().to_token_stream().to_string());
        &mut parse_container.injectable_types_builder.get_mut(&enum_to_add.ident.to_string().clone())
            .map(|struct_impl: &mut Bean| {
                struct_impl.enum_found = Some(enum_to_add.clone());
            })
            .or_else(|| {
                let enum_fields = enum_to_add.variants.iter()
                    .map(|variant| variant.fields.clone())
                    .collect::<Vec<Fields>>();
                let mut impl_found = Bean {
                    struct_type: None,
                    path_depth,
                    struct_found: None,
                    traits_impl: vec![],
                    enum_found: Some(enum_to_add.clone()),
                    attr: vec![],
                    deps_map: vec![],
                    id: enum_to_add.ident.clone().to_string(),
                    profile: vec![],
                    ident: Some(enum_to_add.ident.clone()),
                    fields: enum_fields,
                    bean_type: BeanParser::get_bean_type(&enum_to_add.attrs, None, Some(enum_to_add.ident.clone())),
                    mutable: SynHelper::get_attr_from_vec(&enum_to_add.attrs, vec!["mutable_bean"])
                        .map(|_| true)
                        .or(Some(false)).unwrap(),
                    aspect_info: None,
                };
                parse_container.injectable_types_builder.insert(enum_to_add.ident.to_string().clone(), impl_found);
                None
            });
    }
}

pub struct ItemTraitParser;

impl ItemParser<ItemTrait> for ItemTraitParser {
    fn parse_item(parse_container: &mut ParseContainer, trait_found: &mut ItemTrait, mut path_depth: Vec<String>) {
        path_depth.push(trait_found.ident.to_string().clone());
        if !parse_container.traits.contains_key(&trait_found.ident.to_string().clone()) {
            parse_container.traits.insert(
                trait_found.ident.to_string().clone(),
                Trait::new(trait_found.clone(), path_depth)
            );
        } else {
            log_message!("Contained trait already!");
        }
    }
}
