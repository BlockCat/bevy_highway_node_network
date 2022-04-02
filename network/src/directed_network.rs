use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
    ops::Deref,
};

use serde::{Deserialize, Serialize};

use crate::{BackwardNeighbourhood, ForwardNeighbourhood};

use self::iterators::{BackwardDijkstraIterator, ForwardDijkstraIterator, F32};

pub trait NetworkNode: Send + Sync {
    fn id(&self) -> NodeId;
}

pub trait NetworkEdge: Send + Sync {
    fn source(&self) -> NodeId;
    fn target(&self) -> NodeId;
    fn distance(&self) -> f32;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct NodeId(pub usize);

impl From<usize> for NodeId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl Deref for NodeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct EdgeId(pub usize);

impl From<usize> for EdgeId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl Deref for EdgeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectedNetworkGraph<V: NetworkNode, E: NetworkEdge> {
    pub nodes: Vec<V>,
    pub edges: Vec<E>,
    pub out_edges: Vec<Vec<EdgeId>>,
    pub in_edges: Vec<Vec<EdgeId>>,
}

impl<V: NetworkNode, E: NetworkEdge> DirectedNetworkGraph<V, E> {
    pub fn forward_iterator(&self, node: NodeId) -> ForwardDijkstraIterator<'_, V, E> {
        let mut heap = BinaryHeap::new();
        heap.push(Reverse((F32(0.0), node, None)));
        ForwardDijkstraIterator {
            distance: 0.0,
            network: self,
            heap,
            visited: HashSet::new(),
        }
    }

    pub fn backward_iterator(&self, node: NodeId) -> BackwardDijkstraIterator<'_, V, E> {
        let mut heap = BinaryHeap::new();
        heap.push(Reverse((F32(0.0), node, None)));
        BackwardDijkstraIterator {
            distance: 0.0,
            network: self,
            heap,
            visited: HashSet::new(),
        }
    }

    pub fn forward_neighbourhood(&self, size: usize) -> ForwardNeighbourhood {
        ForwardNeighbourhood::from_network(size, self)
    }

    pub fn backward_neighbourhood(&self, size: usize) -> BackwardNeighbourhood {
        BackwardNeighbourhood::from_network(size, self)
    }
}

pub mod iterators {
    use crate::{DirectedNetworkGraph, EdgeId, NetworkEdge, NetworkNode, NodeId};
    use std::{
        cmp::Reverse,
        collections::{BinaryHeap, HashSet},
    };

    #[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
    pub struct F32(pub f32);

    impl Ord for F32 {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    impl Eq for F32 {}

    pub struct ForwardDijkstraIterator<'a, V: NetworkNode, E: NetworkEdge> {
        pub network: &'a DirectedNetworkGraph<V, E>,
        pub distance: f32,
        pub visited: HashSet<NodeId>,
        pub heap: BinaryHeap<Reverse<(F32, NodeId, Option<EdgeId>)>>,
    }

    pub struct BackwardDijkstraIterator<'a, V: NetworkNode, E: NetworkEdge> {
        pub distance: f32,
        pub network: &'a DirectedNetworkGraph<V, E>,
        pub visited: HashSet<NodeId>,
        pub heap: BinaryHeap<Reverse<(F32, NodeId, Option<EdgeId>)>>,
    }

    impl<'a, V: NetworkNode, E: NetworkEdge> Iterator for ForwardDijkstraIterator<'a, V, E> {
        type Item = (NodeId, f32, Option<EdgeId>);

        fn next(&mut self) -> Option<Self::Item> {
            while let Some(Reverse((F32(distance), node, edge))) = self.heap.pop() {
                if !self.visited.insert(node) {
                    continue;
                }
                for edge_id in &self.network.out_edges[*node] {
                    let edge = &self.network.edges[**edge_id];
                    let target = edge.target();
                    let edge_distance = edge.distance();

                    self.heap.push(Reverse((
                        F32(distance + edge_distance),
                        target,
                        Some(*edge_id),
                    )));
                }

                self.distance = distance;

                return Some((node, distance, edge));
            }
            return None;
        }
    }

    impl<'a, V: NetworkNode, E: NetworkEdge> Iterator for BackwardDijkstraIterator<'a, V, E> {
        type Item = (NodeId, f32, Option<EdgeId>);

        fn next(&mut self) -> Option<Self::Item> {
            while let Some(Reverse((F32(distance), node, edge))) = self.heap.pop() {
                if !self.visited.insert(node) {
                    continue;
                }
                for edge_id in &self.network.in_edges[*node] {
                    let edge = &self.network.edges[**edge_id];
                    let source = edge.source();
                    let edge_distance = edge.distance();

                    self.heap.push(Reverse((
                        F32(distance + edge_distance),
                        source,
                        Some(*edge_id),
                    )));
                }

                self.distance = distance;

                return Some((node, distance, edge));
            }
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{DirectedNetworkGraph, EdgeId, NetworkEdge, NetworkNode, NodeId};

    #[derive(Debug, Clone, Copy)]
    struct TestNode(usize);

    struct TestEdge(usize, usize, f32);

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
    fn create_network() -> DirectedNetworkGraph<TestNode, TestEdge> {
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
                TestEdge(1, 3, 12.0), // B -> D | 2
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

    #[test]
    fn test_forward() {
        let network = create_network();
        let forward = network.forward_iterator(0.into()).collect::<Vec<_>>();

        assert_eq!(
            vec![
                (NodeId(0), 0.0, None),             // A 0.0
                (NodeId(1), 10.0, Some(EdgeId(0))), // B 10.0
                (NodeId(2), 15.0, Some(EdgeId(1))), // C 15.0
                (NodeId(3), 22.0, Some(EdgeId(2))), // D 22
                (NodeId(5), 23.0, Some(EdgeId(6))), // F 23
                (NodeId(4), 24.0, Some(EdgeId(5))), // E 24
            ],
            forward
        );
    }
    #[test]
    fn test_forward_empty() {
        let network = create_network();
        let forward = network.forward_iterator(4.into()).collect::<Vec<_>>();

        assert_eq!(vec![(NodeId(4), 0.0, None)], forward);
    }

    #[test]
    fn test_backward() {
        let network = create_network();
        let backward = network.backward_iterator(4.into()).collect::<Vec<_>>();

        assert_eq!(
            vec![
                (NodeId(4), 0.0, None),
                (NodeId(3), 2.0, Some(EdgeId(5))),
                (NodeId(5), 5.0, Some(EdgeId(7))),
                (NodeId(2), 10.0, Some(EdgeId(4))),
                (NodeId(1), 14.0, Some(EdgeId(2))),
                (NodeId(0), 24.0, Some(EdgeId(0))),
            ],
            backward
        );
    }

    #[test]
    fn test_backward_empty() {
        let network = create_network();
        let backward = network.backward_iterator(0.into()).collect::<Vec<_>>();

        assert_eq!(vec![(NodeId(0), 0.0, None)], backward);
    }
}
