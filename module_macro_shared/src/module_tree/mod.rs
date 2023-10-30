use syn::ItemTrait;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
use crate::logger_lazy;
import_logger!("parse_container.rs");

#[derive(Default, Clone)]
pub struct Trait {
    pub trait_type: Option<ItemTrait>,
    pub trait_path: Vec<String>
}

impl Trait {
    pub fn new(trait_type: ItemTrait, path: Vec<String>) -> Self {
        Self {
            trait_type: Some(trait_type),
            trait_path: path,
        }
    }
}
