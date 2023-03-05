use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn open_syn_file(base_env: &str, lib_file: &str) -> Option<syn::File> {
    open_file(base_env, lib_file)
        .map(|mut b| parse_syn_file(&mut b))
        .ok()
        .flatten()
}

pub fn open_factories_file_syn() -> Option<syn::File> {
    open_syn_file_from_env("KNOCKOFF_FACTORIES")
}

pub fn open_syn_file_from_env(key: &str) -> Option<syn::File> {
    env::var(key)
        .map(|knockoff_factory| {
            parse_syn_from_filename(knockoff_factory)
        })
        .ok()
        .flatten()
}

pub fn parse_syn_from_filename(filename: String) -> Option<syn::File> {
    parse_syn_file(&mut File::open(filename)
        .expect("Could not open knockoff factories file"))
}

pub fn parse_syn_file(file: &mut File) -> Option<syn::File> {
    let mut src = String::new();
    file.read_to_string(&mut src)
        .unwrap();
    syn::parse_file(&src).ok()
}

pub fn open_file(base_env: &str, lib_file: &str) -> Result<File, std::io::Error> {
    File::open(
        Path::new(base_env)
            .join(lib_file)
    )
}
