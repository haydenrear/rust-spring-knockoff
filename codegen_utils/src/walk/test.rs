use crate::walk::DirectoryWalker;

#[test]
fn test_directory_walker() {
    test_walk("test_library_three", "/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test");
    test_walk("test_library_four/test", "/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test");
    test_walk("test_library_five.rs", "/Users/hayde/IdeaProjects/rust-spring-knockoff/delegator_test");
}

fn test_walk(name: &str, dir: &str) {
    let item = DirectoryWalker::walk_directory(name, dir);
    assert_ne!(item.len(), 0);
    assert!(item[0].to_str().unwrap().contains(name))
}