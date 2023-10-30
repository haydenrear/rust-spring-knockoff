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

use module_macro_shared::module_tree::Trait;
use module_macro_shared::parse_container::ParseContainer;

use module_macro_shared::bean::BeanDefinition;
use module_macro_shared::dependency::DependencyDescriptor;
use module_macro_shared::functions::ModulesFunctions;
use module_macro_shared::profile_tree::ProfileBuilder;
use crate::module_macro_lib::util::ParseUtil;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("item_parser.rs");


pub mod item_impl_parser;
pub mod item_enum_parser;
pub mod item_struct_parser;
pub mod item_mod_parser;
pub mod item_trait_parser;
pub mod item_fn_parser;

pub trait ItemParser<T: ToTokens> {
    fn parse_item(parse_container: &mut ParseContainer, item: &mut T, path_depth: Vec<String>);
    fn is_bean(attrs: &Vec<Attribute>) -> bool {
        ParseUtil::does_attr_exist(&attrs, &ParseUtil::get_qualifier_attr_names())
    }
}

fn get_profiles(attrs: &Vec<Attribute>) -> Vec<ProfileBuilder> {
    let mut profiles = SynHelper::get_attr_from_vec(attrs, &vec!["profile"])
        .map(|profile| profile.split(",").map(|s| s.to_string()).collect::<Vec<String>>())
        .or(Some(vec![]))
        .unwrap()
        .iter()
        .map(|profile| ProfileBuilder {profile: profile.replace(" ", "")})
        .collect::<Vec<ProfileBuilder>>();
    profiles.push(ProfileBuilder::default());
    profiles
}
