#![test]

use alloc::string::{String, ToString};

use crate::path::Path;

fn test_path() {
    let mut root = Path::root();
    assert_eq!(root.to_string(), String::from("/"));

    let sub_path = Path::from("a/b/c");
    assert_eq!(sub_path.to_string(), String::from("a/b/c"));

    root.append(&sub_path);
    assert_eq!(root.to_string(), String::from("/a/b/c"));
}