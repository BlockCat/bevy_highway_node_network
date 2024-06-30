use graph::EdgeId;

#[test]
fn edge_from_usize_test() {
    let a: EdgeId = EdgeId::from(0usize);
    let b = EdgeId::from(1usize);
    assert_eq!(*a, 0);
    assert_eq!(*b, 1);
}

#[test]
fn shortcut_into() {
    use graph::ShortcutState;
    let a = ShortcutState::Single(1);
    let b = ShortcutState::Shortcut(vec![1, 2, 3]);
    let c: Vec<i32> = a.into();
    let d: Vec<i32> = b.into();
    assert_eq!(c, vec![1]);
    assert_eq!(d, vec![1, 2, 3]);
}
