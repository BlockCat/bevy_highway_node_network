use crate::{DirectedNetworkGraph, EdgeId, NetworkEdge, NetworkNode, NodeId};
use std::hash::Hash;

#[macro_export]
macro_rules! create_network {
    ($($a:literal => $b:literal; $c: expr),+) => {
    {
        use std::collections::{HashSet, HashMap};
        use crate::{NetworkNode, NetworkEdge};

        let mut nodes = HashSet::new();
        let mut edges = Vec::new();
        $(
            nodes.insert(TestNode($a));
            nodes.insert(TestNode($b));

            edges.push(TestEdge($a, $b, $c));
        )+

        let mut nodes = nodes.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|x| x.0);

        let mut out_edges = HashMap::<NodeId, Vec<EdgeId>>::new();
        let mut in_edges = HashMap::<NodeId, Vec<EdgeId>>::new();

        for edge in edges.iter().enumerate() {
            out_edges
                .entry(edge.1.source())
                .or_default()
                .push(EdgeId(edge.0));
            in_edges
                .entry(edge.1.target())
                .or_default()
                .push(EdgeId(edge.0));
        }

        let out_edges = nodes.iter().map(|x| out_edges.get(&x.id()).cloned().unwrap_or_default()).collect();
        let in_edges = nodes.iter().map(|x| in_edges.get(&x.id()).cloned().unwrap_or_default()).collect();

        DirectedNetworkGraph {
            nodes,
            edges,
            out_edges,
            in_edges,
        }
    }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TestNode(pub usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestEdge(pub usize, pub usize, pub f32);

impl Eq for TestEdge {}

impl Hash for TestEdge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
        state.write_usize((self.2 * 1000.0) as usize);
    }
}

impl NetworkNode for TestNode {
    fn id(&self) -> crate::NodeId {
        self.0.into()
    }
}

impl NetworkEdge for TestEdge {
    fn source(&self) -> crate::NodeId {
        self.0.into()
    }

    fn target(&self) -> crate::NodeId {
        self.1.into()
    }

    fn distance(&self) -> f32 {
        self.2
    }
}

// https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
fn create_ref_network() -> DirectedNetworkGraph<TestNode, TestEdge> {
    DirectedNetworkGraph {
        nodes: vec![
            TestNode(0), // A
            TestNode(1), // B
            TestNode(2), // C
            TestNode(3), // D
            TestNode(4), // E
            TestNode(5), // F
        ],
        edges: vec![
            TestEdge(0, 1, 10.0), // A -> B | 0
            TestEdge(0, 2, 15.0), // A -> C | 1
            TestEdge(1, 3, 5.0),  // B -> D | 2
            TestEdge(1, 5, 15.0), // B -> F | 3
            TestEdge(2, 4, 10.0), // C -> E | 4
            TestEdge(3, 4, 2.0),  // D -> E | 5
            TestEdge(3, 5, 1.0),  // D -> F | 6
            TestEdge(5, 4, 5.0),  // F -> E | 7
        ],
        out_edges: vec![
            vec![EdgeId(0), EdgeId(1)],
            vec![EdgeId(2), EdgeId(3)],
            vec![EdgeId(4)],
            vec![EdgeId(5), EdgeId(6)],
            vec![],
            vec![EdgeId(7)],
        ],
        in_edges: vec![
            vec![],                                // A
            vec![EdgeId(0)],                       // B
            vec![EdgeId(1)],                       // C
            vec![EdgeId(2)],                       // D
            vec![EdgeId(4), EdgeId(5), EdgeId(7)], // E
            vec![EdgeId(3), EdgeId(6)],            // F
        ],
    }
}

pub fn create_undirected_network() -> DirectedNetworkGraph<TestNode, TestEdge> {
    create_network!(
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
    let reference_network = create_ref_network();
    let network = create_network!(
        0 => 1; 10.0,
        0 => 2; 15.0,
        1 => 3; 5.0,
        1 => 5; 15.0,
        2 => 4; 10.0,
        3 => 4; 2.0,
        3 => 5; 1.0,
        5 => 4; 5.0
    );

    assert_eq!(reference_network.nodes, network.nodes);
    assert_eq!(reference_network.edges, network.edges);
    assert_eq!(reference_network.out_edges, network.out_edges);
    assert_eq!(reference_network.in_edges, network.in_edges);
}
