use self::{
    builder::EdgeDirection,
    iterators::{BackwardDijkstraIterator, EdgeIterator, ForwardDijkstraIterator, F32},
};
use crate::{BackwardNeighbourhood, EdgeId, ForwardNeighbourhood, NodeId};
pub use node_data::NetworkData;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
};

pub mod builder;
pub mod iterators;
pub mod node_data;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub id: u32,
    pub start_edge_index: u32,
    pub last_edge_index: u32,
}

impl NetworkNode {
    pub fn new(id: u32, start_edge_index: u32, last_edge_index: u32) -> Self {
        Self {
            id,
            start_edge_index,
            last_edge_index,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NetworkEdge {
    pub edge_id: u32,
    target_node: NodeId,
    edge_weight: f32,
    direction: EdgeDirection,
}

impl Eq for NetworkEdge {}

impl NetworkEdge {
    pub fn new(
        data_id: u32,
        target_node: NodeId,
        edge_weight: f32,
        direction: EdgeDirection,
    ) -> Self {
        Self {
            edge_id: data_id,
            target_node,
            edge_weight,
            direction,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectedNetworkGraph<D: NetworkData = ()> {
    pub data: D,
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

    pub fn edge_data(&self, edge: EdgeId) -> &D::EdgeData {
        self.data.edge_data(edge)
    }

    pub fn node_data(&self, node: NodeId) -> &D::NodeData {
        self.data.node_data(node)
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

    fn create_iterator(&self, node: NodeId, direction: EdgeDirection) -> EdgeIterator {
        self.create_iterator_raw(self.node(node), direction)
    }

    fn create_iterator_raw(&self, node: &NetworkNode, direction: EdgeDirection) -> EdgeIterator {
        let edges =
            self.edges[node.start_edge_index as usize..node.last_edge_index as usize].iter();

        EdgeIterator::new(
            node.start_edge_index..node.last_edge_index,
            edges,
            direction,
        )
    }

    pub fn out_edges(&self, node: NodeId) -> EdgeIterator {
        self.create_iterator(node, EdgeDirection::Forward)
    }

    pub fn out_edges_raw(&self, node: &NetworkNode) -> EdgeIterator {
        self.create_iterator_raw(node, EdgeDirection::Forward)
    }

    pub fn in_edges(&self, node: NodeId) -> EdgeIterator {
        self.create_iterator(node, EdgeDirection::Backward)
    }

    pub fn in_edges_raw(&self, node: &NetworkNode) -> EdgeIterator {
        self.create_iterator_raw(node, EdgeDirection::Backward)
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
