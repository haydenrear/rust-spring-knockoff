use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use proc_macro2::TokenStream;
use syn::parse2;
use syn::token::Use;
use knockoff_logging::error;
use module_macro_codegen::parser::CodegenItem;
use module_macro_shared::profile_tree::ProfileTree;
use knockoff_providers_gen::{DelegatingTokenProvider, DelegatingFrameworkTokenProvider};
use module_macro_shared::ProfileTreeFrameworkTokenProvider;

use module_macro_shared::token_provider::ProfileTreeTokenProvider;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::{build_dir, project_directory};
use crate::logger_lazy;
import_logger!("token_stream_generator.rs");

pub trait TokenStreamGenerator {
    fn generate_token_stream(&self) -> TokenStream;
}

pub trait FrameworkTokenStreamGenerator {
    fn generate_framework_token_stream(&self);
}

pub struct UserProvidedTokenStreamGenerator {
    handler_mapping_token_provider: DelegatingTokenProvider,
    framework_token_provider: DelegatingFrameworkTokenProvider,
}

impl UserProvidedTokenStreamGenerator {
    pub(crate) fn new(profile_tree: &mut ProfileTree) -> Self {
        let handler_mapping_token_provider = DelegatingTokenProvider::new(profile_tree);
        let framework_token_provider = ProfileTreeFrameworkTokenProvider::new(profile_tree);
        Self {
            handler_mapping_token_provider,
            framework_token_provider
        }
    }
}

impl TokenStreamGenerator for UserProvidedTokenStreamGenerator {
    fn generate_token_stream(&self) -> TokenStream {
        self.generate_framework_token_stream();
        self.handler_mapping_token_provider.generate_token_stream()
    }
}

impl FrameworkTokenStreamGenerator for UserProvidedTokenStreamGenerator {
    fn generate_framework_token_stream(&self) {
        // let out_path = Path::new(concat!(build_dir!(), env!("OUT_CODEGEN_DIR")));
        // info!("Writing to out path {:?}", out_path);
        // let mut codegen: Result<File, std::io::Error>;
        // if out_path.exists() {
        //     codegen = File::open(out_path);
        // } else {
        //     codegen = File::create(out_path)
        // }
        // if codegen.is_ok() {
        //     let mut codegen = codegen.unwrap();
        //     let tokens = self.framework_token_provider.generate_token_stream();
        //     let _ = codegen.write(tokens.to_string().as_bytes())
        //         .map_err(|e| {
        //             error!("Error writing codegen.rs: {:?}", e);
        //         });
        // }
    }
}
