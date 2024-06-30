use graph::NodeId;

#[test]
fn node_from_u32_test() {
    let a = NodeId::from(0u32);
    let b = NodeId::from(1u32);
    assert_eq!(*a, 0);
    assert_eq!(*b, 1);
}

