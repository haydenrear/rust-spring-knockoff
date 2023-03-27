use crate::env::get_build_project_dir;
use crate::walk::DirectoryWalker;

#[test]
fn test_directory_walker() {
    let delegator_test = get_build_project_dir("delegator_test").as_str();
    test_walk("test_library_three", delegator_test);
    test_walk("test_library_four/test", delegator_test);
    test_walk("test_library_five.rs", delegator_test);
}

fn test_walk(name: &str, dir: &str) {
    let item = DirectoryWalker::walk_directory(name, dir);
    assert_ne!(item.len(), 0);
    assert!(item[0].to_str().unwrap().contains(name))
}