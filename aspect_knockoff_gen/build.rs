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
    // This will parse the entire program and then generate a crate that will be imported into the
    //  codegen. It will call a dependency crate that will do the following:
    //   go to the knockoff_dfactory and check to see what priority this crate is, creating a
    //   crate of name knockoff_dfactory_{priority_number}.
    //  Then when generating knockoff_providers_gen they will be included to delegate to.
}

