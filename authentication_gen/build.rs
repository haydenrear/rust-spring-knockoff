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
import_logger_root!("build.rs", concat!(project_directory!(), "/log_out/authentication_gen_build.log"));

/// TODO: load the knockoff_factories from here, parse the pre_compiled example.
fn main() {
    info!("Initializing authentication gen build.");
    let generated: TokenStream = module_precompile::get_tokens(&"authentication_gen");
    let out_file = open_out_file();
    if let Some(_) = out_file
        .map(|mut out_file| {
            out_file.write_all(generated.to_string().as_bytes())
                .map_err(|e| {
                    error!("Error writing authentication gen codegen: {:?}", e);
                })
                .ok()
        })
        .map_err(|e| {
            if generated.to_string().as_str().len() != 0 {
                panic!("Could not create codegen.")
            }
            Err(e)
        })
        .ok()
        .flatten() {
        info!("Wrote codegen for authentication gen.");
    } else {
        error!("Failed to write codegen with authentication gen.");
    }
}

fn open_out_file() -> Result<File, std::io::Error> {
    let out_file = concat!(env!("OUT_DIR"), "/codegen.rs");
    let mut out_path = Path::new(out_file);
    let mut out_file = File::create(out_path);
    out_file
}
