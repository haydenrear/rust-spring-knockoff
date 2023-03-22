use std::{env, fs};
use codegen_utils::env::{get_project_base_dir, get_project_dir};
use codegen_utils::project_directory;
use crate_gen::CrateWriter;

fn main() {
    CrateWriter::write_dummy_crate(concat!(project_directory!(), "target/knockoff_providers_gen/Cargo.toml"), "knockoff_providers_gen", "".to_string());
    println!("cargo:rerun-if-changed={}", project_directory!());
}