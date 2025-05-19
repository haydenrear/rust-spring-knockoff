use proc_macro2::TokenStream;

// Type alias for proc_macro2::TokenStream to distinguish from proc_macro::TokenStream
type TokenStream2 = proc_macro2::TokenStream;
use module_macro_shared::profile_tree::ProfileTree;

use codegen_utils::project_directory;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

import_logger_root!("lib.rs", concat!(project_directory!(), "/log_out/boot_knockoff.log"));


pub struct BootKnockoffBuilder {
}

impl BootKnockoffBuilder {
    pub fn new(items: &mut ProfileTree) -> Self {
        Self {}
    }
    
    pub fn generate_token_stream(&self) -> TokenStream {
        TokenStream::default()
    }
}