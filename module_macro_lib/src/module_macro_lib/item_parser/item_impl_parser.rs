use syn::{Attribute, ItemImpl};
use codegen_utils::syn_helper::SynHelper;
use quote::ToTokens;
use std::ops::Deref;
use crate::module_macro_lib::item_parser::ItemParser;
use crate::module_macro_lib::module_tree::{AutowireType, Bean, Profile};
use crate::module_macro_lib::parse_container::ParseContainer;

pub struct ItemImplParser;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;

impl ItemImplParser{
    fn add_path(path_depth: &mut Vec<String>, impl_found: &ItemImpl) {
        let mut trait_impl = vec![];
        impl_found.trait_.clone().map(|trait_found| {
            trait_impl.push(trait_found.1.to_token_stream().to_string());
        });
        trait_impl.push(impl_found.self_ty.to_token_stream().to_string().clone());
        path_depth.push(trait_impl.join("|"));
    }

    fn get_profile(attrs: &Vec<Attribute>) -> Option<String> {
        SynHelper::get_attr_from_vec(attrs, vec!["profile"])
    }

    fn get_qualifier(attrs: &Vec<Attribute>) -> Option<String> {
        SynHelper::get_attr_from_vec(attrs, vec!["qualifier"])
    }

}

impl ItemParser<ItemImpl> for ItemImplParser {
    fn parse_item(parse_container: &mut ParseContainer, item_impl: &mut ItemImpl, mut path_depth: Vec<String>) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        log_message!("Doing create update impl.");

        Self::add_path(&mut path_depth, &item_impl);

        let profile = Self::get_profile(&item_impl.attrs)
            .map(|profile| profile.split(", ")
                .map(|profile| profile.to_string())
                .map(|mut profile| profile.replace(" ", ""))
                .map(|profile| Profile {profile})
                .collect::<Vec<Profile>>()
            )
            .or(Some(vec![]))
            .unwrap();

        let qualifiers = Self::get_qualifier(&item_impl.attrs)
            .map(|profile| profile.split(", ")
                .map(|profile| profile.to_string())
                .map(|mut profile| profile.replace(" ", ""))
                .collect::<Vec<String>>()
            )
            .or(Some(vec![]))
            .unwrap();

        &mut parse_container.injectable_types_builder.get_mut(&id)
            .map(|bean: &mut Bean| {
                bean.traits_impl.push(
                    AutowireType {
                        item_impl: item_impl.clone(),
                        profile: profile.clone(),
                        path_depth: path_depth.clone(),
                        qualifiers: qualifiers.clone()
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
                            profile: profile.clone(),
                            path_depth: path_depth.clone(),
                            qualifiers: qualifiers.clone()
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
                        .or(Some(false))
                        .unwrap(),
                    aspect_info: vec![],
                };
                parse_container.injectable_types_builder.insert(id.clone(), impl_found);
                None
            });

        log_message!("Adding method advice aspect now.");

        // let aspect_modifier = AspectModifier{};
        // aspect_modifier.add_method_advice_aspect(parse_container, item_impl, &mut path_depth, &id);
    }
}

impl ItemImplParser {

    pub fn get_trait(item_impl: &mut ItemImpl) -> Option<syn::Path> {
        item_impl.trait_.clone()
            .and_then(|item_impl_found| {
                Some(item_impl_found.1)
            })
            .or_else(|| None)
    }

}
