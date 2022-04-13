use crate::{
    builder::{EdgeBuilder, EdgeDirection, NodeBuilder},
    DirectedNetworkGraph, NetworkEdge, NetworkNode, NodeId,
};
use std::hash::Hash;

#[macro_export]
macro_rules! create_network {
    ($s:literal..$e:literal, $($a:literal => $b:literal; $c: expr),+) => {
    {
        let mut builder = crate::builder::DirectedNetworkBuilder::<TestNode, TestEdge>::new();


        for x in $s..=$e {
            builder.add_node(TestNode(x));
        }

        $({
            let source = builder.add_node(TestNode($a));
            let target = builder.add_node(TestNode($b));

            builder.add_edge(TestEdge::forward(source, target, $c));

        })+

        builder.build::<()>()
    }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TestNode(pub usize);

impl NodeBuilder for TestNode {
    type Data = ();

    fn data(&self) -> Self::Data {
        ()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestEdge(pub NodeId, pub NodeId, pub f32, EdgeDirection);

impl Eq for TestEdge {}

impl Hash for TestEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
        state.write_usize((self.2 * 1000.0) as usize);
    }
}

impl EdgeBuilder for TestEdge {
    type Data = ();

    fn data(&self) -> Self::Data {
        ()
    }

    fn source(&self) -> NodeId {
        self.0.into()
    }

    fn target(&self) -> NodeId {
        self.1.into()
    }

    fn weight(&self) -> f32 {
        self.2
    }

    fn direction(&self) -> crate::builder::EdgeDirection {
        self.3
    }

    fn road_id(&self) -> crate::ShortcutState<usize> {
        todo!()
    }
}

impl TestEdge {
    pub fn forward(source: NodeId, target: NodeId, weight: f32) -> Self {
        Self(source, target, weight, EdgeDirection::Forward)
    }
}

// https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
pub fn create_ref_network_1() -> DirectedNetworkGraph<()> {
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
