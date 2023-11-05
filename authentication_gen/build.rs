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
    if let Some(written) = out_file
        .map(|mut out_file| {
            out_file.write_all(generated.to_string().as_bytes())
                .map_err(|e| {
                    error!("Error writing authentication gen codegen: {:?}", e);
                })
                .map(|e| generated.to_string())
                .ok()
        })
        .flatten()
        .or_else(|| {
            if generated.to_string().as_str().len() != 0 {
                panic!("Could not create codegen.")
            }
            None
        }) {
        info!("Wrote codegen for authentication gen: {:?}.", &written);
    } else {
        error!("Failed to write codegen with authentication gen.");
    }
}

fn open_out_file() -> Option<File> {
    std::env::var("OUT_DIR")
        .map_err(|o| {
            error!("Out directory was not defined: {:?}", o);
        })
        .ok()
        .map(|out_file| Path::new(&out_file).join("codegen.rs"))
        .map(|path| File::create(path)
            .map_err(|e| {
                error!("Error creating path: {:?}", e);
            })
            .ok()
        )
        .flatten()
}
