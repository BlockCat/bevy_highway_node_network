use self::iterators::{BackwardDijkstraIterator, ForwardDijkstraIterator, F32};
use crate::{BackwardNeighbourhood, ForwardNeighbourhood};
pub use node_data::NetworkData;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
    ops::Deref,
};

pub mod builder;
pub mod iterators;
pub mod node_data;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    forward_edge_index: u32,
    backward_edge_index: u32,
    both_edge_index: u32,
    last_edge_index: u32,
}

impl NetworkNode {
    pub fn new(
        forward_edge_index: u32,
        both_edge_index: u32,
        backward_edge_index: u32,
        last_edge_index: u32,
    ) -> Self {
        Self {
            forward_edge_index,
            backward_edge_index,
            both_edge_index,
            last_edge_index,
        }
    }

    pub fn out_len(&self) -> usize {
        let forward = self.forward_edge_index as usize;
        let end = self.last_edge_index as usize;

        end - forward
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NetworkEdge {
    // id: EdgeId,
    target_node: NodeId,
    edge_weight: f32,
}

impl Eq for NetworkEdge {}

impl NetworkEdge {
    pub fn new(target_node: NodeId, edge_weight: f32) -> Self {
        Self {
            // id,
            target_node,
            edge_weight,
        }
    }
    // pub fn id(&self) -> EdgeId {
    //     self.id
    // }
    pub fn target(&self) -> NodeId {
        self.target_node.into()
    }

    pub fn distance(&self) -> f32 {
        self.edge_weight
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl From<usize> for NodeId {
    fn from(id: usize) -> Self {
        Self(id as u32)
    }
}

impl Deref for NodeId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for NodeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct EdgeId(pub u32);

impl From<usize> for EdgeId {
    fn from(id: usize) -> Self {
        Self(id as u32)
    }
}

impl From<u32> for EdgeId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl Deref for EdgeId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectedNetworkGraph<D: NetworkData = ()> {
    data: D,
    nodes: Vec<NetworkNode>,
    edges: Vec<NetworkEdge>,
}

impl<D: NetworkData> DirectedNetworkGraph<D> {
    pub fn new(nodes: Vec<NetworkNode>, edges: Vec<NetworkEdge>, data: D) -> Self {
        Self { data, nodes, edges }
    }

    pub fn node(&self, node: NodeId) -> &NetworkNode {
        &self.nodes[node.0 as usize]
    }

    pub fn nodes(&self) -> &Vec<NetworkNode> {
        &self.nodes
    }

    pub fn edge(&self, edge: EdgeId) -> &NetworkEdge {
        &self.edges[edge.0 as usize]
    }

    pub fn edges(&self) -> &Vec<NetworkEdge> {
        &self.edges
    }

    pub fn out_edges(
        &self,
        node: NodeId,
    ) -> std::iter::Zip<std::slice::Iter<NetworkEdge>, std::ops::Range<u32>> {
        self.out_edges_raw(self.node(node))
    }

    pub fn in_edges(&self, node: NodeId) -> &[NetworkEdge] {
        self.in_edges_raw(self.node(node))
    }

    pub fn out_edges_raw(
        &self,
        node: &NetworkNode,
    ) -> std::iter::Zip<std::slice::Iter<NetworkEdge>, std::ops::Range<u32>> {
        self.edges[node.forward_edge_index as usize..node.backward_edge_index as usize]
            .iter()
            .zip(node.forward_edge_index..node.backward_edge_index)
    }

    pub fn in_edges_raw(&self, node: &NetworkNode) -> &[NetworkEdge] {
        &self.edges[node.both_edge_index as usize..node.last_edge_index as usize]
    }

    pub fn forward_iterator(&self, node: NodeId) -> ForwardDijkstraIterator<'_, D> {
        let mut heap = BinaryHeap::new();
        heap.push(Reverse((F32(0.0), node)));
        ForwardDijkstraIterator {
            distance: 0.0,
            network: self,
            heap,
            visited: HashSet::new(),
        }
    }

    pub fn backward_iterator(&self, node: NodeId) -> BackwardDijkstraIterator<'_, D> {
        let mut heap = BinaryHeap::new();
        heap.push(Reverse((F32(0.0), node)));
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

#[cfg(test)]
mod tests {
    use crate::{tests::create_ref_network_1, NodeId};

    #[test]
    fn test_forward() {
        let network = create_ref_network_1();
        let forward = network.forward_iterator(0u32.into()).collect::<Vec<_>>();

        assert_eq!(
            vec![
                (NodeId(0), 0.0),  // A 0.0
                (NodeId(1), 10.0), // B 10.0
                (NodeId(2), 15.0), // C 15.0
                (NodeId(3), 22.0), // D 22
                (NodeId(5), 23.0), // F 23
                (NodeId(4), 24.0), // E 24
            ],
            forward
        );
    }
    #[test]
    fn test_forward_empty() {
        let network = create_ref_network_1();
        let forward = network.forward_iterator(4u32.into()).collect::<Vec<_>>();

        assert_eq!(vec![(NodeId(4), 0.0)], forward);
    }

    #[test]
    fn test_backward() {
        let network = create_ref_network_1();
        let backward = network.backward_iterator(4u32.into()).collect::<Vec<_>>();

        assert_eq!(
            vec![
                (NodeId(4), 0.0),
                (NodeId(3), 2.0),
                (NodeId(5), 5.0),
                (NodeId(2), 10.0),
                (NodeId(1), 14.0),
                (NodeId(0), 24.0),
            ],
            backward
        );
    }

    #[test]
    fn test_backward_empty() {
        let network = create_ref_network_1();
        let backward = network.backward_iterator(0u32.into()).collect::<Vec<_>>();

        assert_eq!(vec![(NodeId(0), 0.0)], backward);
    }
}
