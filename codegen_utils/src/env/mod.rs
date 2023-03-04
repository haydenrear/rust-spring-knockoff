use std::env;

pub fn get_project_base_dir() -> String {
    env::var("PROJECT_BASE_DIRECTORY")
        .ok()
        .or(Some("/Users/hayde/IdeaProjects/rust-spring-knockoff/".to_string()))
        .unwrap()
}

pub fn get_project_dir(path: &str) -> String {
    let mut base = env::var("PROJECT_BASE_DIRECTORY")
        .ok()
        .or(Some("/Users/hayde/IdeaProjects/rust-spring-knockoff/".to_string()))
        .unwrap();
    base += path;
    base
}

#[macro_export]
macro_rules! project_directory {
    () => {
        env!("PROJECT_BASE_DIRECTORY", "Please set project base directory.")
    };
}
