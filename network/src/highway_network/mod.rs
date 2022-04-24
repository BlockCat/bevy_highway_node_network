use serde::{Deserialize, Serialize};

use crate::{builder::EdgeDirection, DirectedNetworkGraph, NodeId};

use self::iterators::EdgeIterator;

mod iterators;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct HighwayNetworkBaseNode {
    start_edge_index: u32,
    last_edge_index: u32,
    start_node_index: u32,
    end_node_index: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct HighwayNetworkLevelNode {
    start_edge_index: u32,
    last_edge_index: u32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NetworkEdge {
    pub edge_id: u32,
    target_node: NodeId,
    edge_weight: f32,
    direction: EdgeDirection,
}

impl Eq for NetworkEdge {}

pub struct HighwayNetwork {
    nodes: Vec<HighwayNetworkBaseNode>,
    highway_nodes: Vec<HighwayNetworkLevelNode>,
    edges: Vec<NetworkEdge>,
}

/**
 * Requirements:
 * - Node
 * - Is node in layer
 * - Is node bypassed
 * - Node forward/backward neighbourhood radius
 * - Node starts at edge_index
 * - Edge
 * - Is edge in layer
 * - Is edge bypassed in layer...
 * - weight
 */

impl HighwayNetwork {
    pub fn new(base: DirectedNetworkGraph, layers: Vec<DirectedNetworkGraph>) -> Self {
        unimplemented!()
    }

    pub fn node(&self, node: NodeId) -> &HighwayNetworkBaseNode {
        &self.nodes[node.0 as usize]
    }

    pub fn node_level(&self, node: NodeId, level: u8) -> Option<&HighwayNetworkLevelNode> {
        assert!(level > 0);
        let node = self.node(node);
        let levels = node.end_node_index - node.start_node_index;

        if level as u32 > levels {
            return None;
        }

        self.highway_nodes
            .get(node.start_node_index as usize + level as usize)
    }

    pub fn out_edges(&self, node: NodeId, level: u8) -> Option<EdgeIterator> {
        EdgeIterator::new(node, level, EdgeDirection::Forward, self)
    }

    pub fn in_edges(&self, node: NodeId, level: u8) -> Option<EdgeIterator> {
        EdgeIterator::new(node, level, EdgeDirection::Backward, self)
    }
}
