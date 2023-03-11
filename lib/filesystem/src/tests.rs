use crate::path::{Component, Path};

#[test]
fn test_path_buf() {
    let mut components_iter = Path::default().components();
    assert_eq!(components_iter.next(), Some(Component::Root));
    assert_eq!(components_iter.next(), None);
}