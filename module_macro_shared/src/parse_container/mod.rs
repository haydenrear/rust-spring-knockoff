use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use syn::{Item, ItemImpl, ItemMod, Type};
use quote::ToTokens;
use crate::bean::BeanDefinition;
use crate::functions::{FunctionType, ModulesFunctions};
use crate::module_tree::Trait;
use crate::profile_tree::{ProfileBuilder, ProfileTree};

use knockoff_logging::*;
use std::sync::Mutex;
use codegen_utils::FlatMapOptional;
use codegen_utils::syn_helper::SynHelper;
use crate::{DefaultItemModifier, DefaultProfileTreeFinalizer, ItemModifier, logger_lazy, ProfileTreeFinalizer};
import_logger!("parse_container.rs");


pub trait MetadataItem: 'static + Debug {
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Ord, PartialEq, Hash, Eq, PartialOrd, Clone, Debug)]
pub struct MetadataItemId {
    pub item_id: String,
    pub metadata_item_type_id: String
}

impl MetadataItemId {
    pub fn new(item_id: String, metadata_item_type_id: String) -> Self {
        Self {
            item_id, metadata_item_type_id
        }
    }
}

#[derive(Default)]
pub struct ParseContainer {
    pub injectable_types_builder: HashMap<String, BeanDefinition>,
    pub profile_tree: ProfileTree,
    pub traits: HashMap<String, Trait>,
    pub fns: HashMap<String, ModulesFunctions>,
    pub profiles: Vec<ProfileBuilder>,
    pub provided_items: HashMap<MetadataItemId, Vec<Box<dyn MetadataItem>>>
}

impl ParseContainer {
    // pub fn get_item_impl<'a>(&'a mut self, item: &mut Item) -> Option<&mut ItemImpl> {
    //     if let Item::Impl(i) = item {
    //         return Some(self.injectable_types_builder.get_mut(&i.self_ty.to_token_stream().to_string())
    //             .map(|bd| bd.traits_impl.iter_mut().flat_map(|s| s.item_impl.as_mut().into_iter())
    //                 .filter(|i| i.self_ty.to_token_stream().to_string().as_str() == i.self_ty.to_token_stream().to_string().as_str()).next())
    //             .flatten()
    //             .or(Some(i))
    //             .unwrap());
    //     }
    //
    //     None
    //
    //
    // }

    pub fn get_bean_definition_key(i: &Item) -> Option<String> {
        match &i {
            Item::Enum(e) => Some(e.ident.to_string().clone()),
            Item::Fn(fn_) => Some(fn_.sig.ident.to_string().clone()),
            Item::Impl(imp) => Some(imp.self_ty.deref().to_token_stream().to_string().clone()),
            Item::Mod(m) => Some(m.ident.to_string().clone()),
            Item::Static(s) => Some(s.ident.to_string().clone()),
            Item::Struct(s) => Some(s.ident.to_string().clone()),
            Item::Trait(t) => Some(t.ident.to_string().clone()),
            Item::Type(ty) => Some(ty.ident.to_string().clone()),
            _ => None
        }
    }
}

impl Debug for ParseContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        log_message!("hello");
        Ok(())
    }
}

impl ParseContainer {

    pub fn get_injectable_keys(&self) -> Vec<String> {
        self.injectable_types_builder.keys().map(|k| k.clone()).collect()
    }

    pub fn log_app_container_info(&self) {
        self.injectable_types_builder.iter().filter(|&s| s.1.struct_found.as_ref().is_none() && s.1.ident.as_ref().is_none())
            .for_each(|s| {
                log_message!("Could not find struct type with ident {}.", s.0.clone());
            })
    }


    pub fn get_type_from_fn_type(fn_type: &FunctionType) -> Option<Type> {
        fn_type.fn_type.as_ref()
            .map(|f| f.get_inner_type())
            .flatten()
    }
}


/// ItemModifier runs as the ParseContainer is loaded with the beans. It is running at the same
/// time as the code in module_macro_lib.item_parser
pub trait ParseContainerItemUpdater {
    fn parse_update(items: &mut Item, parse_container: &mut ParseContainer);
}

#[derive(Default)]
pub struct DefaultParseContainerItemUpdater;
impl ParseContainerItemUpdater for DefaultParseContainerItemUpdater { fn parse_update(items: &mut Item, parse_container: &mut ParseContainer) {} }

pub trait ParseContainerModifier {
    fn do_modify(items: &mut ParseContainer);
}

#[derive(Default)]
pub struct DefaultParseContainerModifier;
impl ParseContainerModifier for DefaultParseContainerModifier { fn do_modify(items: &mut ParseContainer) {} }

pub trait BuildParseContainer {
    fn build_parse_container(&self, parse_container: &mut ParseContainer);
}

#[derive(Default)]
pub struct DefaultBuildParseContainer;
impl BuildParseContainer for DefaultBuildParseContainer { fn build_parse_container(&self, parse_container: &mut ParseContainer) {} }


/// After the
/// 1. ParseContainerItemUpdater and the
/// 2. ItemModifier run
/// the final build is done, and
/// so the
/// 3. ParseContainerModifier is passed here to perform any finalizing changes.
/// 4. BuildParseContainer is used to build the parse container to the profile tree
/// 5. ProfileTreeFinalizer is used as a hook after the profile tree has been created
///
/// Then there is a final TokenStreamGenerator hook that is in the module_macro_lib and FrameworkTokenStreamGenerator
/// that is in the pre_compile lib that are not contained here, because they are for different
/// codegen phases.
pub struct ModuleParser<
    ParseContainerItemUpdaterT,
    ItemModifierT,
    ParseContainerModifierT,
    BuildParseContainerT,
    ParseContainerFinalizerT
>
    where
        ParseContainerItemUpdaterT: ParseContainerItemUpdater,
        ItemModifierT: ItemModifier,
        ParseContainerModifierT: ParseContainerModifier,
        BuildParseContainerT: BuildParseContainer,
        ParseContainerFinalizerT: ProfileTreeFinalizer,
{
    pub delegating_parse_container_updater: ParseContainerItemUpdaterT,
    pub delegating_parse_container_modifier: ParseContainerModifierT,
    pub delegating_parse_container_builder: BuildParseContainerT,
    pub delegating_parse_container_item_modifier: ItemModifierT ,
    pub delegating_parse_container_finalizer: ParseContainerFinalizerT
}

pub type DefaultModuleParser = ModuleParser<DefaultParseContainerItemUpdater, DefaultItemModifier, DefaultParseContainerModifier, DefaultBuildParseContainer, DefaultProfileTreeFinalizer>;

pub fn get_test_module_parser() -> DefaultModuleParser {
    DefaultModuleParser {
        delegating_parse_container_updater: Default::default(),
        delegating_parse_container_modifier: Default::default(),
        delegating_parse_container_builder: Default::default(),
        delegating_parse_container_item_modifier: Default::default(),
        delegating_parse_container_finalizer: Default::default(),
    }
}