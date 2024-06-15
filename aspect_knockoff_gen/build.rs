use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use proc_macro2::TokenStream;

use knockoff_logging::*;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use codegen_utils::project_directory;
import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/aspect_knockoff_gen.log"));

/// TODO: load the knockoff_factories from here, parse the pre_compiled example.
fn main() {
    info!("Initializing aspect knockoff gen build.");
    let generated: TokenStream = module_precompile::get_tokens(&"aspect_knockoff_gen");
    let out_file = codegen_utils::parse::open_out_file("codegen.rs");
    codegen_utils::parse::write(out_file, generated.to_string().as_str(), "aspect_knockoff_gen");
}

