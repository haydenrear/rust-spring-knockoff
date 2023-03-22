use std::env;

#[macro_export]
macro_rules! project_directory {
    () => {
        env!("PROJECT_BASE_DIRECTORY", "Please set project base directory.")
    };
}


pub fn get_project_base_dir() -> String {
   project_directory!().to_string()
}

pub fn get_project_dir(path: &str) -> String {
    let mut project_dir = project_directory!().to_string();
    project_dir += path;
    project_dir
}
