use graph::{create_network, BackwardNeighbourhood, ForwardNeighbourhood, NodeId};

#[test]
fn forward_test() {
    // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
    let network = create_network!(
        0..5,
        0 => 1; 10.0, // A => B
        0 => 2; 15.0, // A => C
        1 => 3; 5.0, // B => D
        1 => 5; 15.0, // B => F
        2 => 4; 10.0, // C => E
        3 => 4; 2.0, // D => E
        3 => 5; 1.0, // D => F
        5 => 4; 5.0 // F => E
    );

    let forward = ForwardNeighbourhood::from_network(3, &network);

    assert_eq!(forward.radius(NodeId(0)), 15.0);
    assert_eq!(forward.radius(NodeId(1)), 6.0);
    assert_eq!(forward.radius(NodeId(2)), 10.0);
    assert_eq!(forward.radius(NodeId(3)), 2.0);
    assert_eq!(forward.radius(NodeId(4)), 0.0);
    assert_eq!(forward.radius(NodeId(5)), 5.0);
}



#[test]
fn backward_test() {
    // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
    let network = create_network!(
        0..5,
        0 => 1; 10.0, // A => B
        0 => 2; 15.0, // A => C
        1 => 3; 5.0, // B => D
        1 => 5; 15.0, // B => F
        2 => 4; 10.0, // C => E
        3 => 4; 2.0, // D => E
        3 => 5; 1.0, // D => F
        5 => 4; 5.0 // F => E
    );

    let forward = BackwardNeighbourhood::from_network(3, &network);

    assert_eq!(forward.radius(NodeId(0)), 0.0);
    assert_eq!(forward.radius(NodeId(1)), 10.0);
    assert_eq!(forward.radius(NodeId(2)), 15.0);
    assert_eq!(forward.radius(NodeId(3)), 15.0);
    assert_eq!(forward.radius(NodeId(4)), 5.0);
    assert_eq!(forward.radius(NodeId(5)), 6.0);
}
