use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::logger_lazy;
import_logger!("parse.rs");

pub fn open_file(base_env: &str, lib_file: &str) -> Result<File, std::io::Error> {
    open_file_from_path(
        &Path::new(base_env)
            .join(lib_file)
    )
}

pub fn open_file_from_path(lib_file: &PathBuf) -> Result<File, std::io::Error> {
    File::open(lib_file)
        .map_err(|e| {
            error!("Failed to open file from path {:?}", &lib_file.to_str());
            e
        })
}

pub fn open_file_from_str(lib_file: &str) -> Result<File, std::io::Error> {
    open_file_from_path(&Path::new(lib_file).to_path_buf())
}


pub fn read_path_to_str(file: &PathBuf) -> Result<String, std::io::Error> {
    let opened_read =
     open_file_from_path(file)
        .map(|mut f| {
            read_file_to_str(&f)
        });

    if opened_read.is_ok() {
        opened_read.unwrap()
    } else {
        Err::<String, std::io::Error>(opened_read.err().unwrap())
    }
}

pub fn read_file_to_str(mut f: &File) -> Result<String, std::io::Error> {
    let mut out_str = String::default();
    let e = f.read_to_string(&mut out_str);
    e.map_err(|e| {
            error!("Error reading {:?}", e);
            e
        })
        .map(|s| out_str)
}


pub fn read_path_to_bytes<'a>(file: &PathBuf, in_bytes: &'a mut [u8]) -> Result<&'a [u8], std::io::Error> {
    let opened = open_file_from_path(file);
    if opened.as_ref().is_ok() {
        let mut f = opened.unwrap();
        let opened_read = read_file_to_bytes(&f, in_bytes);
        if opened_read.is_ok() {
            Ok(opened_read.unwrap())
        } else {
            Err::<&[u8], std::io::Error>(opened_read.err().unwrap())
        }
    } else {
        Err::<&[u8], std::io::Error>(opened.err().unwrap())
    }
}

pub fn read_file_to_bytes<'a>(mut f: &File, in_bytes: &'a mut [u8]) -> Result<&'a mut [u8], std::io::Error> {
    let e = f.read(in_bytes);
    e.map_err(|e| {
        error!("Error reading {:?}", e);
        e
    }).map(|s| in_bytes)
}

pub fn open_out_file(path: &str) -> Option<File> {
    std::env::var("OUT_DIR")
        .map_err(|o| {
            error!("Out directory was not defined: {:?}", o);
        })
        .ok()
        .map(|out_file| Path::new(&out_file).join(path))
        .map(|path| File::create(path)
            .map_err(|e| {
                error!("Error creating path: {:?}", e);
            })
            .ok()
        )
        .flatten()
}

pub fn write(mut out_file: Option<File>, generated: &str, name: &str) {

    if let Some(written) = out_file
        .map(|mut out_file| {
            out_file.write_all(generated.to_string().as_bytes())
                .map_err(|e| {
                    error!("Error writing {} gen: {:?}", name, e);
                })
                .map(|e| generated.to_string())
                .ok()
        })
        .flatten()
        .or_else(|| {
            if generated.to_string().as_str().len() != 0 {
                panic!("Could not create {}.", name)
            }
            None
        }) {
        info!("Wrote codegen for {} gen: {:?}.", name, &written);
    } else {
        error!("Failed to write {}.", name);
    }
}