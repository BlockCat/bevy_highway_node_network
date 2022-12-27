use crate::{
    builder::EdgeDirection, create_network, DirectedNetworkGraph, NetworkEdge, NetworkNode,
};

// https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
pub fn create_ref_network_1() -> DirectedNetworkGraph<()> {
    let nodes = vec![
        NetworkNode::new(0, 0, 2),
        NetworkNode::new(1, 2, 5),
        NetworkNode::new(2, 5, 7),
        NetworkNode::new(3, 7, 10),
        NetworkNode::new(4, 10, 13),
        NetworkNode::new(5, 13, 16),
    ];
    let edges = vec![
        NetworkEdge::new(0, 1u32.into(), 10.0, EdgeDirection::Forward), // A -> B
        NetworkEdge::new(1, 2u32.into(), 15.0, EdgeDirection::Forward), // A -> C
        NetworkEdge::new(2, 3u32.into(), 12.0, EdgeDirection::Forward), // B -> D
        NetworkEdge::new(3, 5u32.into(), 15.0, EdgeDirection::Forward), // B -> F
        NetworkEdge::new(4, 0u32.into(), 10.0, EdgeDirection::Backward), // A <- B
        NetworkEdge::new(5, 4u32.into(), 10.0, EdgeDirection::Forward), // C -> E
        NetworkEdge::new(6, 0u32.into(), 15.0, EdgeDirection::Backward), // A <- C
        NetworkEdge::new(7, 4u32.into(), 2.0, EdgeDirection::Forward),  // D -> E
        NetworkEdge::new(8, 5u32.into(), 1.0, EdgeDirection::Forward),  // D -> F
        NetworkEdge::new(9, 1u32.into(), 12.0, EdgeDirection::Backward), // B <- D
        NetworkEdge::new(10, 2u32.into(), 10.0, EdgeDirection::Backward), // C <- E
        NetworkEdge::new(11, 3u32.into(), 2.0, EdgeDirection::Backward), // D <- E
        NetworkEdge::new(12, 5u32.into(), 5.0, EdgeDirection::Backward), // F <- E
        NetworkEdge::new(13, 4u32.into(), 5.0, EdgeDirection::Forward), // F -> E
        NetworkEdge::new(14, 1u32.into(), 15.0, EdgeDirection::Backward), // B <- F
        NetworkEdge::new(15, 3u32.into(), 1.0, EdgeDirection::Backward), // D <- F
    ];

    DirectedNetworkGraph::new(nodes, edges, ())
}

pub fn create_undirected_network() -> DirectedNetworkGraph<()> {
    create_network!(
        0..16,
        0 => 1; 3.0,
        1 => 0; 3.0,
        1 => 2; 2.0,
        2 => 1; 2.0,
        2 => 3; 2.0,
        3 => 2; 2.0,
        3 => 0; 2.0,
        0 => 3; 2.0,

        7 => 6; 2.0,
        6 => 7; 2.0,
        6 => 5; 2.0,
        5 => 6; 2.0,
        4 => 5; 3.0,
        5 => 4; 3.0,
        4 => 7; 2.0,
        7 => 4; 2.0,

        8 => 9; 3.0,
        9 => 8; 3.0,
        9 => 10; 2.0,
        10 => 9; 2.0,
        10 => 11; 2.0,
        11 => 10; 2.0,
        11 => 8; 2.0,
        8 => 11; 2.0,

        12 => 13; 2.0,
        13 => 12; 2.0,
        13 => 14; 2.0,
        14 => 13; 2.0,
        14 => 15; 3.0,
        15 => 14; 3.0,
        15 => 12; 2.0,
        12 => 15; 2.0,

        12 => 16; 6.0,
        16 => 12; 6.0,
        16 => 0; 5.0,
        0 => 16; 5.0,

        13 => 7; 7.0,
        7 => 13; 7.0,

        2 => 4; 6.0,
        4 => 2; 6.0,

        14 => 11; 14.0,
        11 => 14; 14.0
    )
}

#[test]
fn create_test() {
    let reference_network = create_ref_network_1();
    // Gotta put it in weird order so it sees creation of nodes in correct order, the numbers are labels after all.
    let network = create_network!(
        0..5,
        0 => 1; 10.0,
        0 => 2; 15.0,
        1 => 3; 12.0,
        1 => 5; 15.0,
        2 => 4; 10.0,
        3 => 4; 2.0,
        3 => 5; 1.0,
        5 => 4; 5.0
    );
    assert_eq!(network.nodes(), reference_network.nodes(), "Fails on nodes");
    assert_eq!(network.edges(), reference_network.edges(), "Fails on edges");
    assert_eq!(network, reference_network);
}
