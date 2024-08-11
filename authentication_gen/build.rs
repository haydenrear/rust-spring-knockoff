use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use proc_macro2::TokenStream;

use knockoff_logging::*;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::{program_src, project_directory, user_program_src};

import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/authentication_gen_build.log"));

/// TODO: load the knockoff_factories from here, parse the pre_compiled example.
fn main() {
    /// TODO: This should be externalized into a single delegator just like the others so that it
    ///       only has to iterate over once for all precompile (a new phase).
    info!("Initializing authentication gen build.");
    let generated: TokenStream = module_precompile::get_tokens(&"authentication_gen", &user_program_src!());
    let out_file = codegen_utils::parse::open_out_file("codegen.rs");
    codegen_utils::parse::write(out_file, generated.to_string().as_str(), "authentication_gen");
}

