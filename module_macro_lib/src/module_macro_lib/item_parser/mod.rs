use std::any::Any;
use std::ops::Deref;
use std::path::Path;
use std::thread::available_parallelism;
use paste::item;
use quote::ToTokens;
use syn::{Attribute, Fields, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait};
use codegen_utils::syn_helper::SynHelper;
use item_impl_parser::ItemImplParser;
use module_macro_shared::module_macro_shared_codegen::FieldAugmenter;

use crate::module_macro_lib::logging::StandardLoggingFacade;
use crate::module_macro_lib::logging::executor;
use crate::module_macro_lib::module_tree::{AutowireType, Bean, ModulesFunctions, Profile, Trait};
use crate::module_macro_lib::parse_container::ParseContainer;

use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();

pub mod item_impl_parser;
pub mod item_enum_parser;
pub mod item_struct_parser;
pub mod item_mod_parser;
pub mod item_trait_parser;
pub mod item_fn_parser;

pub trait ItemParser<T: ToTokens> {
    fn parse_item(parse_container: &mut ParseContainer, item: &mut T, path_depth: Vec<String>);
}

fn get_profiles(attrs: &Vec<Attribute>) -> Vec<Profile> {
    let mut profiles = SynHelper::get_attr_from_vec(attrs, vec!["profile"])
        .map(|profile| profile.split(",").map(|s| s.to_string()).collect::<Vec<String>>())
        .or(Some(vec![]))
        .unwrap()
        .iter()
        .map(|profile| Profile {profile: profile.replace(" ", "")})
        .collect::<Vec<Profile>>();
    profiles.push(Profile::default());
    profiles
    // vec![]
}
