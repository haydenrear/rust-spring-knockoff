use std::collections::HashMap;
use syn::{Attribute, Generics, ImplItem, ItemImpl};
use codegen_utils::syn_helper::SynHelper;
use quote::ToTokens;
use std::ops::Deref;
use crate::item_parser::{create_new_gens, GenericTy, get_all_generic_ty_bounds, get_profiles, ItemParser};
use crate::bean::{BeanDefinition, BeanPath};
use crate::parse_container::ParseContainer;

pub struct ItemImplParser;

use crate::dependency::DependencyDescriptor;
use crate::profile_tree::ProfileBuilder;
use crate::bean_parser::bean_dependency_path_parser::BeanDependencyPathParser;
use crate::util::ParseUtil;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use proc_macro2::TokenStream;
use codegen_utils::project_directory;
use crate::{BuildParseContainer, ItemModifier, logger_lazy, ModuleParser, ParseContainerItemUpdater, ParseContainerModifier, ProfileTreeFinalizer};
import_logger!("item_impl_parser.rs");


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
        SynHelper::get_attr_from_vec(attrs, &vec!["profile"])
    }

    fn get_qualifier(attrs: &Vec<Attribute>) -> Option<String> {
        SynHelper::get_attr_from_vec(attrs, &vec!["qualifier"])
    }

    fn get_generics(item_impl: &mut ItemImpl) -> Generics {
        item_impl.generics.clone()
    }

    fn add_bean_defn(parse_container: &mut ParseContainer, item_impl: &mut ItemImpl,
                     mut path_depth: &mut Vec<String>, id: &String, profile: &Vec<ProfileBuilder>,
                     qualifiers: &Vec<String>) {

        let abstract_type = item_impl.trait_.as_ref()
            .map(|trait_impl| BeanDependencyPathParser::parse_path_to_bean_path(&trait_impl.1));



        &mut parse_container.injectable_types_builder.get_mut(id)
            .map(|bean: &mut BeanDefinition| {
                bean.traits_impl.push(
                    DependencyDescriptor {
                        item_impl: Some(item_impl.clone()),
                        abstract_type: abstract_type.clone(),
                        profile: profile.clone(),
                        path_depth: path_depth.clone(),
                        qualifiers: qualifiers.clone(),
                        item_impl_gens: Self::get_generics(item_impl),
                    }
                );
                info!("Added trait to {:?}", bean);
            })
            .or_else(|| {
                let mut impl_found = BeanDefinition {
                    struct_type: Some(item_impl.self_ty.deref().clone()),
                    struct_found: None,
                    traits_impl: vec![
                        DependencyDescriptor {
                            item_impl: Some(item_impl.clone()),
                            abstract_type,
                            profile: profile.clone(),
                            path_depth: path_depth.clone(),
                            qualifiers: qualifiers.clone(),
                            item_impl_gens: Self::get_generics(item_impl),
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
                    mutable: ParseUtil::does_attr_exist(&item_impl.attrs, &vec!["mutable_bean"]),
                    factory_fn: None,
                    declaration_generics: None,
                    qualifiers: vec![],
                };
                info!("Created bean {:?}", &impl_found);
                parse_container.injectable_types_builder.insert(id.clone(), impl_found);
                None
            });
    }
}

pub fn matches_ignore_traits(matches_ignore_traits: &str) -> bool {
    vec!["Default", "Debug", "Serialize", "Deserialize"].iter()
        .any(|i| matches_ignore_traits.contains(i))
}

pub fn is_ignore_trait(item_impl: &ItemImpl) -> bool {
    if SynHelper::get_attr_from_vec(&item_impl.attrs, &vec![
        "knockoff_ignore"
    ]).is_some() {
        true
    } else if item_impl.trait_.as_ref().filter(|t| matches_ignore_traits(&SynHelper::get_str(t.1.to_token_stream().to_string().as_str())))
        .is_some() {
        log_message!("Ignoring {}.", SynHelper::get_str(&item_impl));
        return true;
    } else   {
        false
    }
}

impl ItemParser<ItemImpl> for ItemImplParser {
    fn parse_item<
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
    >(parse_container: &mut ParseContainer, item_impl: &mut ItemImpl, mut path_depth: Vec<String>, module_parser: &mut ModuleParser<
        ParseContainerItemUpdaterT,
        ItemModifierT,
        ParseContainerModifierT,
        BuildParseContainerT,
        ParseContainerFinalizerT
    >) {
        let id = item_impl.self_ty.to_token_stream().to_string().clone();

        log_message!("Doing create update impl for id: {}", id);

        if is_ignore_trait(&item_impl) {
            info!("Ignoring trait: {:?}", SynHelper::get_str(&item_impl));
            return;
        }

        item_impl.trait_.as_ref().map(|t| {
            log_message!("Doing create update impl for trait impl: {}", SynHelper::get_str(&t.1));
        });

        Self::add_path(&mut path_depth, &item_impl);

        let profile = ParseUtil::get_profile(&item_impl.attrs);

        let qualifiers = ParseUtil::get_qualifiers(&item_impl.attrs);

        Self::add_bean_defn(parse_container, item_impl, &mut path_depth, &id, &profile, &qualifiers);
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
