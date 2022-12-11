use std::{env, fs};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;
use syn::parse::{ParseBuffer, ParseStream};

fn main() {
    // replace_modules();
}

//TODO: take all modules declared as not inline and move them in-line to allow for attribute macro to have full access to all
// the types in the modules.
fn replace_modules() {
    // env::var_os("OUT_DIR")
    let mut file = File::open(Path::new(env::current_dir().unwrap().to_str().unwrap()).join("test_library/test_library_two.rs"))
        .unwrap();

    let mut src = String::new();
    file.read_to_string(&mut src);

    let syn_file = syn::parse_file(&src).unwrap();


    let out_path = Path::new(OsString::from("/Users/hayde/IdeaProjects/rust-spring-knockoff").deref()).join("another.rs");

    fs::write(&out_path, env::current_dir().unwrap().to_str().unwrap()).unwrap() ;

    println!("cargo:rerun-if-changed=build.rs");
}