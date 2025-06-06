use module_macro_shared::parse_container::{MetadataItemId, ParseContainer};
use syn::{Item, ItemImpl, ItemMod, ItemStruct};
use knockoff_logging::{info, log_message};
use std::collections::HashMap;
use quote::{quote, ToTokens};
use crate::{AuthTypes, METADATA_ITEM_ID, METADATA_TYPE_ITEM_ID, NextAuthType};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use proc_macro2::TokenStream;
use codegen_utils::project_directory;
use crate::authentication_gen_token_stream_provider::AuthenticationTypeTokenStreamGenerator;
use crate::logger_lazy;
import_logger!("authentication_gen_item_modifier.rs");

pub struct AuthenticationGenItemModifier;

impl AuthenticationGenItemModifier {
    pub fn new() -> Self {
        Self {}
    }

    /// This runs after the parse provider, which means that the aspects from the program have already
    /// been loaded into the ParseContainer. So now the item can be modified to add the aspect.
    pub fn modify_item(parse_container: &mut ParseContainer,
                       item: &mut Item, path_depth: Vec<String>) {
        info!("Testing if modify {}!.", item.to_token_stream().to_string().as_str());
        if Self::supports_item(item) {
            match item {
                Item::Mod(item_mod) => {
                    let item_id = MetadataItemId::new(METADATA_ITEM_ID.into(), METADATA_TYPE_ITEM_ID.into());
                    assert!(parse_container.provided_items.get_mut(&item_id).is_none(), "Authentication gen was already contained in container.");
                    let codegen_item = Self::get_codegen(item_mod);
                    info!("Adding auth types: {:?}.", &codegen_item);
                    parse_container.provided_items.insert(
                        item_id,
                        vec![Box::new(codegen_item)]
                    );
                }
                _ => {
                }
            }
        }
    }

    pub fn supports_item(impl_item: &Item) -> bool{
        info!("Checking {:?}", &impl_item.to_token_stream().to_string().as_str());
        match impl_item {
            Item::Mod(item_mod) => {
                item_mod.attrs.iter()
                    .any(|attr_found| attr_found.to_token_stream()
                        .to_string().as_str().contains("authentication_type")
                    )
            }
            _ => {
                false
            }
        }
    }

    fn get_codegen(item: &ItemMod) -> AuthTypes {
        let mut to_add_map: HashMap<String, NextAuthType> = HashMap::new();
        item.content
            .iter().flat_map(|m| &m.1)
            .for_each(|item| {
                info!("Testing if {}.", item.to_token_stream().to_string().as_str());
                if let Item::Struct(s) = item {
                    Self::add_item_struct(&mut to_add_map, s);
                };
                if let Item::Impl(i) = item {
                    Self::insert_item_impl(&mut to_add_map, &i);
                };

            });
        AuthTypes {
            auth_types: to_add_map.iter().map(|(k, v)| v.clone()).collect()
        }
    }

    fn add_item_impl(mut to_add_map: &mut HashMap<String, NextAuthType>, id: &String, impl_found: &ItemImpl) {
        if impl_found.attrs.iter().any(|a| a.to_token_stream().to_string().as_str().contains("auth_type_impl")) {
            info!("Found auth type");
            to_add_map.get_mut(id).map(|f| f.auth_type_impl = Some(impl_found.clone()));
        }
        if impl_found.attrs.iter().any(|a| a.to_token_stream().to_string().as_str().contains("auth_type_aware")) {
            info!("Found auth aware");
            to_add_map.get_mut(id).map(|f| f.auth_aware_impl = Some(impl_found.clone()));
        }

    }

    fn add_item_struct(mut to_add_map: &mut HashMap<String, NextAuthType>, struct_found: &ItemStruct) {
        struct_found.attrs.iter().for_each(|attr| {
            log_message!("{} is the path.", attr.path.to_token_stream().to_string().as_str());
            log_message!("{} is the other.", attr.tokens.to_token_stream().to_string().as_str());
        });
        let id = struct_found.ident.to_token_stream().to_string().clone();
        let struct_opt_to_add = Some(struct_found.clone());
        info!("Found struct");
        if to_add_map.contains_key(&id) {
            to_add_map.get_mut(&id).map(|f| {
                f.auth_type_to_add = struct_opt_to_add
            });
        } else {
            let next = NextAuthType {
                auth_type_to_add: struct_opt_to_add,
                auth_type_impl: None,
                auth_aware_impl: None,
            };
            to_add_map.insert(id, next);
        }
    }

    fn insert_item_impl(mut to_add_map: &mut HashMap<String, NextAuthType>, impl_found: &&ItemImpl) {
        let id = impl_found.self_ty.clone().to_token_stream().to_string();
        if to_add_map.contains_key(&id) {
            Self::add_item_impl(&mut to_add_map, &id, &impl_found)
        } else {
            to_add_map.insert(id.clone(), NextAuthType {
                auth_type_to_add: None,
                auth_type_impl: None,
                auth_aware_impl: None,
            });
            Self::add_item_impl(&mut to_add_map, &id, &impl_found)
        }
    }

    fn add_item_to_map(mut to_add_map: &mut HashMap<String, NextAuthType>, item_to_create: &Item) {
        match item_to_create {
            Item::Struct(struct_found) => {
                Self::add_item_struct(&mut to_add_map, struct_found);
            }
            Item::Impl(impl_found) => {
                Self::insert_item_impl(&mut to_add_map, &impl_found);
            }
            _ => {}
        }
    }

}
