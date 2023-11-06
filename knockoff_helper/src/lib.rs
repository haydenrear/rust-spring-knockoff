#[macro_export]
macro_rules! project_directory {
    () => {
        env!("PROJECT_BASE_DIRECTORY", "Please set project base directory.")
    };
}

#[macro_export]
macro_rules! project_directory_runtime {
    ($lit:literal) => {
        Path::new(&env::var("PROJECT_BASE_DIRECTORY").unwrap()).join($lit).to_str().unwrap().to_string().as_str()
    };
}

#[macro_export]
macro_rules! build_dir {
    () => {
        concat!(env!("PROJECT_BASE_DIRECTORY", "Please set project base directory."), "/target")
    };
}
use std::path::{Path, PathBuf};
use std::env;

pub fn get_project_base_build_dir() -> String {
    project_directory!().to_string()
}

pub fn get_current_build_base_dir() -> String {
    project_directory!().to_string()
}

pub fn get_build_project_base_path() -> PathBuf {
    Path::new(&project_directory!().to_string()).to_path_buf()
}

pub fn get_build_project_dir(path: &str) -> String {
    let mut project_dir = project_directory!().to_string();
    Path::new(project_dir.as_str()).join(path).to_str().unwrap().to_string()
}

pub fn get_build_project_path(path: &str) -> PathBuf {
    get_build_project_base_path().join(path)
}

pub fn get_project_base_dir() -> String {
    env::var("PROJECT_BASE_DIRECTORY")
        .ok()
        .or_else(|| env::current_dir()
            .map(|c| c.to_str().map(|path| path.to_string()))
            .ok()
            .flatten()
            .or_else(|| Some(get_current_build_base_dir()))
        )
        .unwrap()
}

pub fn get_project_base_path() -> PathBuf {
    Path::new(&get_project_base_dir()).to_path_buf()
}

pub fn get_project_dir(path: &str) -> String {
    get_project_path(path)
        .to_str()
        .or(Some(get_build_project_dir(path).as_str()))
        .map(|path| path.to_string())
        .unwrap()
}

pub fn get_project_path(path: &str) -> PathBuf {
    get_project_base_path().join(path)
}
