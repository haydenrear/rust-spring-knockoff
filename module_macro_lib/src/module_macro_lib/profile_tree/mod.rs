use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use quote::ToTokens;
use module_macro_shared::bean::{BeanDefinition, BeanDefinitionType};
use module_macro_shared::parse_container::{MetadataItem, MetadataItemId};
use module_macro_shared::profile_tree::{ProfileBuilder, ProfileTree};
use module_macro_shared::profile_tree::profile_tree_modifier::ProfileTreeModifier;

use knockoff_logging::*;
use std::sync::Mutex;
use module_macro_shared::dependency::DepType;
use crate::logger_lazy;
import_logger!("profile_tree.rs");


pub mod mutable_profile_tree_modifier;
pub mod concrete_profile_tree_modifier;
pub mod profile_profile_tree_modifier;
pub mod bean_type_profile_tree_modifier;
pub(crate) mod search_profile_tree;

