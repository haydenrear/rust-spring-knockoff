use build_lib::replace_modules;
use codegen_utils::{get_build_project_dir, get_project_base_build_dir};

fn main() {
    replace_modules(
        Some(get_build_project_dir("web_framework/src").as_str()),
        vec![get_project_base_build_dir().as_str()]
    );
}