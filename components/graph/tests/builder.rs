use std::ops::Neg;

use graph::{
    builder::{DefaultEdgeBuilder, DirectedNetworkBuilder, EdgeDirection},
    DirectedNetworkGraph, NetworkEdge, NetworkNode,
};

// https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
fn create_ref_network_1() -> DirectedNetworkGraph<()> {
    let nodes = vec![
        NetworkNode::new(0, 2),
        NetworkNode::new(2, 5),
        NetworkNode::new(5, 7),
        NetworkNode::new(7, 10),
        NetworkNode::new(10, 13),
        NetworkNode::new(13, 16),
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

fn create_network() -> DirectedNetworkGraph<()> {
    let mut builder = DirectedNetworkBuilder::new();

    let na = builder.add_node(0);
    let nb = builder.add_node(1);
    let nc = builder.add_node(2);
    let nd = builder.add_node(3);
    let ne = builder.add_node(4);
    let nf = builder.add_node(5);

    builder.add_edge(DefaultEdgeBuilder::forward(na, nb, 0, 10.0));
    builder.add_edge(DefaultEdgeBuilder::forward(nb, nd, 0, 12.0));
    builder.add_edge(DefaultEdgeBuilder::forward(nd, ne, 0, 2.0));
    builder.add_edge(DefaultEdgeBuilder::forward(na, nc, 0, 15.0));
    builder.add_edge(DefaultEdgeBuilder::forward(nc, ne, 0, 10.0));
    builder.add_edge(DefaultEdgeBuilder::forward(nf, ne, 0, 5.0));
    builder.add_edge(DefaultEdgeBuilder::forward(nb, nf, 0, 15.0));
    builder.add_edge(DefaultEdgeBuilder::forward(nd, nf, 0, 1.0));

    builder.build()
}

#[test]
fn builder_test() {
    let n1 = create_ref_network_1();
    let n2 = create_network();

    assert_eq!(n1, n2);
}

#[test]
fn reverse() {
    assert_eq!(EdgeDirection::Backward.neg(), EdgeDirection::Forward);
    assert_eq!(EdgeDirection::Forward.neg(), EdgeDirection::Backward);
    assert_eq!(EdgeDirection::Both.neg(), EdgeDirection::Both);
}
