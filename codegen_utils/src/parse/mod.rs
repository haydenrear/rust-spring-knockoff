use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn open_file(base_env: &str, lib_file: &str) -> Result<File, std::io::Error> {
    File::open(
        Path::new(base_env)
            .join(lib_file)
    )
}
