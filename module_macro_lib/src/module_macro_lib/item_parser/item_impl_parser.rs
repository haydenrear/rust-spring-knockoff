use syn::{Attribute, ItemImpl};
use codegen_utils::syn_helper::SynHelper;
use quote::ToTokens;
use std::ops::Deref;
use crate::module_macro_lib::item_parser::{get_profiles, ItemParser};
use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::parse_container::ParseContainer;

pub struct ItemImplParser;

use knockoff_logging::{initialize_log, use_logging};
use module_macro_shared::dependency::DependencyDescriptor;
use module_macro_shared::profile_tree::ProfileBuilder;
use_logging!();
initialize_log!();
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::util::ParseUtil;

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

pub fn matches_ignore_traits(matches_ignore_traits: &str) -> bool {
    vec!["Default", "Debug"].iter().any(|i| matches_ignore_traits.contains(i))
}

pub fn is_ignore_trait(item_impl: &ItemImpl) -> bool {
    if item_impl.trait_.as_ref().filter(|t| matches_ignore_traits(&SynHelper::get_str(t.1.to_token_stream().to_string().as_str())))
        .is_some() {
        log_message!("Ignoring {}.", SynHelper::get_str(&item_impl));
        return true;
    }
    false
}

impl ItemParser<ItemImpl> for ItemImplParser {
    fn parse_item(parse_container: &mut ParseContainer, item_impl: &mut ItemImpl, mut path_depth: Vec<String>) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();
        log_message!("Doing create update impl for id: {}", id);
        item_impl.trait_.as_ref().map(|t| {
            log_message!("Doing create update impl for trait impl: {}", SynHelper::get_str(&t.1));
        });

        if is_ignore_trait(&item_impl) {
            return;
        }

        Self::add_path(&mut path_depth, &item_impl);

        let profile = ParseUtil::get_profile(&item_impl.attrs);

        let qualifiers = ParseUtil::get_qualifiers(&item_impl.attrs);

        &mut parse_container.injectable_types_builder.get_mut(&id)
            .map(|bean: &mut BeanDefinition| {
                bean.traits_impl.push(
                    DependencyDescriptor {
                        item_impl: Some(item_impl.clone()),
                        abstract_type: None,
                        profile: profile.clone(),
                        path_depth: path_depth.clone(),
                        qualifiers: qualifiers.clone()
                    }
                );
            })
            .or_else(|| {
                let mut impl_found = BeanDefinition {
                    struct_type: Some(item_impl.self_ty.deref().clone()),
                    struct_found: None,
                    traits_impl: vec![
                        DependencyDescriptor {
                            item_impl: Some(item_impl.clone()),
                            abstract_type: None,
                            profile: profile.clone(),
                            path_depth: path_depth.clone(),
                            qualifiers: qualifiers.clone()
                        }
                    ],
                    enum_found: None,
                    deps_map: vec![],
                    id: id.clone(),
                    path_depth: vec![],
                    profile: get_profiles(&item_impl.attrs),
                    ident: None,
                    fields: vec![],
                    bean_type: None,
                    mutable: ParseUtil::does_attr_exist(&item_impl.attrs, vec!["mutable_bean"]),
                    aspect_info: vec![],
                    factory_fn: None,
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
