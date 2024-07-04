use self::iterators::EdgeIterator;
use crate::{
    dijkstra_iterator::DijkstraIterator, Backward, EdgeId, Forward, Neighbourhood, NodeId,
};
use serde::{Deserialize, Serialize};
use std::ops::Neg;

pub mod iterators;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EdgeDirection {
    Forward,
    Both,
    Backward,
}

impl Neg for EdgeDirection {
    type Output = EdgeDirection;

    fn neg(self) -> Self::Output {
        match self {
            EdgeDirection::Forward => EdgeDirection::Backward,
            EdgeDirection::Both => EdgeDirection::Both,
            EdgeDirection::Backward => EdgeDirection::Forward,
        }
    }
}

/// A node in the graph.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    /// The index of the first edge in the list of edges that are connected to this node.
    pub start_edge_index: u32,
    /// The index of the last edge in the list of edges that are connected to this node.
    pub last_edge_index: u32,
}

impl NetworkNode {
    pub fn new(start_edge_index: u32, last_edge_index: u32) -> Self {
        Self {
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

    pub fn weight(&self) -> f32 {
        self.edge_weight
    }
}

/// A Directed network graph, the graph is represented by a list of nodes and a list of edges.
/// It's an adjacency list representation of a graph.
/// The graph is immutable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectedNetworkGraph {
    nodes: Vec<NetworkNode>,
    edges: Vec<NetworkEdge>,
}

impl DirectedNetworkGraph {
    pub fn new(nodes: Vec<NetworkNode>, edges: Vec<NetworkEdge>) -> Self {
        Self { nodes, edges }
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

    pub fn forward_iterator(&self, node: NodeId) -> DijkstraIterator<'_, Forward> {
        DijkstraIterator::new(self, node)
    }

    pub fn backward_iterator(&self, node: NodeId) -> DijkstraIterator<'_, Backward> {
        DijkstraIterator::new(self, node)
    }

    pub fn forward_neighbourhood(&self, size: usize) -> Neighbourhood<Forward> {
        Neighbourhood::from_network(size, self)
    }

    pub fn backward_neighbourhood(&self, size: usize) -> Neighbourhood<Backward> {
        Neighbourhood::from_network(size, self)
    }
}
