use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::string::ToString;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use syn::{ItemImpl, ItemStruct};
use codegen_utils::project_directory;
use module_macro_shared::impl_parse_values;

import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/authentication_gen.log"));


pub mod authentication_gen_token_stream_provider;
pub use authentication_gen_token_stream_provider::*;

pub mod authentication_gen_item_modifier;
pub use authentication_gen_item_modifier::*;


pub const METADATA_ITEM_ID: &'static str = "AuthType";
pub const METADATA_TYPE_ITEM_ID: &'static str = "AuthenticationType";


#[derive(Default, Clone)]
struct NextAuthType {
    auth_type_to_add: Option<ItemStruct>,
    auth_type_impl: Option<ItemImpl>,
    auth_aware_impl: Option<ItemImpl>,
}

#[derive(Default, Clone)]
pub struct AuthTypes {
    auth_types: Vec<NextAuthType>
}

impl Debug for AuthTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthTypes")?;
        f.write_str(format!("auth_type_to_add: {}", self.auth_types.len()).as_str())?;
        Ok(())
    }
}

use module_macro_shared::parse_container::MetadataItem;

impl_parse_values!(AuthTypes);

impl MetadataItem for AuthTypes {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
