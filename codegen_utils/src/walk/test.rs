use knockoff_helper::get_build_project_dir;
use crate::walk::DirectoryWalker;

use knockoff_logging::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use radix_trie::TrieCommon;
use crate::logger_lazy;
import_logger!("walk.rs");

#[test]
fn test_directory_walker() {
    let string = get_build_project_dir("delegator_test/src");
    let delegator_test = string.as_str();
    test_walk("test_library_three", delegator_test);
    let mut string1 = get_build_project_dir("");
    let item = DirectoryWalker::walk_directories_matching(
        &|w| DirectoryWalker::file_name_matches(
            w, &|p| true),
        &|w| true,
        string1.as_str()
    );
    info!("Walked and found {} values in trie", item.len());
}

#[test]
fn walk_find_mod() {
    let string = get_build_project_dir("delegator_test/src");
    let b = string.as_str();
    println!("Searching {} for test_library_five", b);
    let found = DirectoryWalker::walk_find_mod(
        "test_library_five",
        b
    );
    assert_eq!(found.len(), 1);
    let found = DirectoryWalker::walk_find_mod(
        "test_library_three",
        b
    );
    assert_eq!(found.len(), 1);
    let found = DirectoryWalker::walk_find_mod(
        "test_library_seven",
        b
    );
    assert_eq!(found.len(), 1);
}

fn test_walk(name: &str, dir: &str) {
    let item = DirectoryWalker::walk_find_mod(name, dir);
    assert_eq!(item.len(), 1);
    info!("Found items {:?}", &item);
    assert!(item[0].to_str().unwrap().contains(name))
}