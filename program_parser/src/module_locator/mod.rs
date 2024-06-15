use std::fs::File;
use std::path::PathBuf;
use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use syn::{Ident, ItemMod, Path};
use syn::__private::Span;
use codegen_utils::{env, project_directory, program_src};
use codegen_utils::syn_helper::SynHelper;
use codegen_utils::walk::DirectoryWalker;
use crate::logger_lazy;
use codegen_utils::FlatMapOptional;
use properties::Properties;
import_logger!("module_locator.rs");

pub fn get_module_from_module_name(module_name: &Ident) -> Option<(PathBuf, Result<File, std::io::Error>)> {
    let base_directory = get_path();
    get_module_from_name_base(&base_directory, module_name)
}

pub fn get_module_from_name_base(base_dir: &PathBuf, module_name: &Ident) -> Option<(PathBuf, Result<File, std::io::Error>)> {
    base_dir.parent()
        .flat_map_opt(|parent| get_module_from_name(base_dir, &parent.to_path_buf(), module_name))
}

pub fn is_in_line_module(inline_module: &ItemMod) -> bool {
    info!("Checking if {:?} is inline", SynHelper::get_str(inline_module.clone()));
    if inline_module.content.as_ref().is_none() || inline_module.content.as_ref().unwrap().1.len() == 0 {
        info!("Found module in file: {:?}", SynHelper::get_str(inline_module.clone()));
        false
    } else {
        let (brace, items) = inline_module.content.as_ref().unwrap();
        items.len() == 0
    }
}

pub fn get_module_from_name(base_dir: &PathBuf, parent_buf: &PathBuf, module_name: &Ident) -> Option<(PathBuf, Result<File, std::io::Error>)> {
    info!("Searching for {} with base {}", module_name, base_dir.as_os_str().to_str().unwrap());
    let dirs_with_name = DirectoryWalker::walk_find_mod(SynHelper::get_str(module_name).as_str(), &base_dir);

    if dirs_with_name.len() == 1 {
        info!("Found one dirs {:?} when walking.", &dirs_with_name[0].to_str().as_ref().unwrap());
        let correct_buf = dirs_with_name[0].to_owned();
        return Some((correct_buf.clone(), File::open(correct_buf)));
    } else if dirs_with_name.len() == 0 {
        info!("Did not find any dir when walking to find {} from parent {:?}", base_dir.as_os_str().to_str().unwrap(), module_name.clone());
        return None
    } else {
        let paths = dirs_with_name.iter()
            .flat_map(|d| d)
            .flat_map(|d| d.to_str().into_iter().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        info!("Found multiple dirs when walking: {:?}",
            paths);
    }

    find_correct_buf(dirs_with_name, parent_buf)
        .map(|correct_buf| Some((correct_buf.clone(), File::open(correct_buf))))
        .flatten()
        .or(None)
}

fn find_correct_buf(bufs: Vec<PathBuf>, parent_buf: &PathBuf) -> Option<PathBuf> {
    bufs.iter().filter(|b| {
        parent_buf.to_str()
            .map(|parent_buf| b.to_str().map(|child_buf| parent_buf.contains(child_buf)))
            .or_else(|| panic!("Parent path could not be unwrapped!"))
            .is_some()
    })
        .max_by(|first, second| {
            (first.to_str().unwrap().len() as i32).cmp(&(second.to_str().unwrap().len() as i32))
        })
        .map(|c| c.to_owned())
        .map(|c| {
            info!("Found next: {:?}", c.to_str());
            c
        })
}

#[test]
fn test_find_module() {
    let base_directory = get_path();
    let found = get_module_from_name_base(&base_directory, &Ident::new("first_module", Span::call_site())).unwrap();
    println!("{}", found.0.to_str().unwrap());
    let found = get_module_from_name_base(&base_directory, &Ident::new("second", Span::call_site())).unwrap();
    println!("{}", found.0.to_str().unwrap());
    let found = get_module_from_name_base(&base_directory, &Ident::new("third", Span::call_site())).unwrap();
    println!("{}", found.0.to_str().unwrap());
    let found = get_module_from_name_base(&base_directory, &Ident::new("fourth", Span::call_site())).unwrap();
    println!("{}", found.0.to_str().unwrap());
    let found = get_module_from_name_base(&base_directory, &Ident::new("fifth", Span::call_site())).unwrap();
    println!("{}", found.0.to_str().unwrap());
    let found = get_module_from_name_base(&base_directory, &Ident::new("sixth", Span::call_site())).unwrap();
    println!("{}", found.0.to_str().unwrap());
    let found = get_module_from_name_base(&base_directory, &Ident::new("seventh", Span::call_site())).unwrap();
    println!("{}", found.0.to_str().unwrap());
    let found = get_module_from_name_base(&base_directory, &Ident::new("ninth", Span::call_site()));
    println!("{}", found.is_none());
    let module_path_val = module_path!();
    println!("{}", module_path_val);
}
pub fn get_path_from(name: Option<&str>) -> PathBuf {
    let proj = program_src!(name.unwrap());
    PathBuf::from(proj)
}

pub fn get_path() -> PathBuf {
    let proj = program_src!("test");
    PathBuf::from(proj)
}