use syn::__private::{str, TokenStream, ToTokens};
use syn::Attribute;
use std::env;
use std::fs::File;
use std::path::Path;
use std::io::Read;
use std::fmt::{Debug, DebugStruct};
use crate::parse;

pub mod test;

pub struct SynHelper;

impl SynHelper {

    pub fn parse_attr_path_single(attr: &Attribute) -> Option<String> {
        attr.tokens.to_string().strip_suffix(")")
            .map(|stripped_suffix| stripped_suffix.strip_prefix("("))
            .flatten()
            .map(|stripped| stripped.to_string())
    }

    pub fn get_attr_from_vec(autowired_attr: &Vec<Attribute>, matcher_str: Vec<&str>) -> Option<String> {
        autowired_attr.iter()
            .filter(|a| matcher_str.iter().any(|m| Self::get_str(a).as_str().contains(*m)))
            .next()
            .map(|a| SynHelper::parse_attr_path_single(a).or(Some("".to_string())))
            .flatten()
    }

    pub fn get_attr_from_vec_ref(autowired_attr: &Vec<&Attribute>, matcher_str: &Vec<&str>) -> Option<String> {
        autowired_attr.iter()
            .filter(|a| matcher_str.iter().any(|m| Self::get_str(a).as_str().contains(*m)))
            .next()
            .map(|a| SynHelper::parse_attr_path_single(a).or(Some("".to_string())))
            .flatten()
    }

    pub fn get_str<'a, T: ToTokens>(ts: T) -> String {
        ts.to_token_stream().to_string().clone()
    }

    pub fn get_proceed(name: String) -> String {
        let name = name.split("proceed").collect::<Vec<&str>>();
        let name = name[1].split("(").collect::<Vec<&str>>();
        let name = name[0].to_owned();
        name
    }


    pub fn open_syn_file(base_env: &str, lib_file: &str) -> Option<syn::File> {
        parse::open_file(base_env, lib_file)
            .map(|mut b| Self::parse_syn_file(&mut b))
            .ok()
            .flatten()
    }

    pub fn open_factories_file_syn() -> Option<syn::File> {
        Self::open_syn_file_from_env("AUG_FILE")
    }

    pub fn open_syn_file_from_env(key: &str) -> Option<syn::File> {
        env::var(key)
            .map(|knockoff_factory| {
                Self::parse_syn_from_filename(knockoff_factory)
            })
            .ok()
            .flatten()
    }

    pub fn parse_syn_from_filename(filename: String) -> Option<syn::File> {
        Self::parse_syn_file(&mut File::open(filename)
            .expect("Could not open knockoff factories file"))
    }

    pub fn parse_syn_file(file: &mut File) -> Option<syn::File> {
        let mut src = String::new();
        file.read_to_string(&mut src)
            .unwrap();
        syn::parse_file(&src).ok()
    }

    pub fn open_from_base_dir(file_name_path: &str) -> syn::File {
        Self::parse_syn_file(
            &mut env::var("PROJECT_BASE_DIRECTORY")
                .map(|p| {
                    Path::new(&p).join(file_name_path)
                })
                .map(|p| {
                    File::open(p).expect("Could not open factories file")
                })
                .ok()
                .expect("Could not open factories file")
        ).expect("Could not parse syn file.")
    }


}

pub fn debug_struct_field_opt<T: ToString>(debug_struct: &mut DebugStruct, field: &Option<T>, field_name: &str) {
    field.as_ref().map(|f| debug_struct.field(field_name, &f.to_string().as_str()));
}

pub fn debug_struct_field_opt_tokens<T: ToTokens>(debug_struct: &mut DebugStruct, field: &Option<T>, field_name: &str) {
    field.as_ref().map(|f|  f.to_token_stream())
        .map(|t| debug_struct.field(field_name, &t.to_string().as_str()));
}

fn write_optional_struct_field<T: ToTokens>(name: &str, f: &mut DebugStruct, optional: &Option<T>) {
    optional.as_ref().map(|opt| f.field(name, &opt.to_token_stream().to_string().as_str()));
}

pub fn debug_struct_vec_field_tokens<T: ToTokens>(name: &str, f: &mut DebugStruct, optional: &Vec<T>) {
    for opt in optional.iter() {
        f.field(name, &opt.to_token_stream().to_string().as_str());
    }
}

fn debug_struct_vec_field_string<T: ToTokens>(name: &str, f: &mut DebugStruct, optional: &Vec<T>) {
    for opt in optional.iter() {
        f.field(name, &opt.to_token_stream().to_string().as_str());
    }
}

pub fn debug_debug_struct_field_opt<T: Debug>(debug_struct: &mut DebugStruct, field: &Option<T>, field_name: &str) {
    field.as_ref().map(|f| debug_struct.field(field_name, &f));
}

pub fn debug_struct_vec_field_debug<T: Debug>(name: &str, f: &mut DebugStruct, optional: &Vec<T>) {
    for opt in optional.iter() {
        f.field(name, opt);
    }
}

