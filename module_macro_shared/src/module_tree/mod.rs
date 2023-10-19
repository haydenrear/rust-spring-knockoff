use syn::ItemTrait;
use knockoff_logging::{initialize_log, use_logging};
use_logging!();
initialize_log!();
use crate::logger::executor;
use crate::logger::StandardLoggingFacade;

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
