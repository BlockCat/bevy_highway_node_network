use std::collections::HashMap;

use itertools::Itertools;
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct HighwayNetworkLevelNode {
    start_edge_index: u32,
    last_edge_index: u32,
    pub forward_radius: f32,
    pub backward_radius: f32,
    pub is_bypassed: bool,
}

impl Eq for HighwayNetworkLevelNode {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct NetworkEdge {
    pub edge_id: u32,
    target_node: NodeId,
    edge_weight: f32,
    level: u8,
    pub direction: EdgeDirection,
}

impl Eq for NetworkEdge {}

impl NetworkEdge {
    pub fn target(&self) -> NodeId {
        self.target_node
    }

    pub fn distance(&self) -> f32 {
        self.edge_weight
    }

    pub fn level(&self) -> u8 {
        self.level
    }
}

pub struct HighwayNetwork {
    pub max_level: u8,
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
 * - Is edge bypassed or is a shortcut (starting from which layer)
 * - weight
 */
impl HighwayNetwork {
    pub fn new(base: DirectedNetworkGraph, layers: Vec<DirectedNetworkGraph>) -> Self {
        // unimplemented!()
        // Level 0  is base,
        // Level >1 are layers
        // let max_level = layers.len() as u8;
        // let mut nodes = Vec::with_capacity(base.nodes().len());
        // let mut highway_nodes = Vec::with_capacity(base.nodes().len() * (max_level / 2u8));
        // let mut edges = Vec::with_capacity(base.edges().len());

        let mut collect_mapper = HashMap::new();

        for (id, node) in base.nodes().into_iter().enumerate() {
            let out_edges = base.out_edges_raw(node).collect_vec();
            let in_edges = base.in_edges_raw(node).collect_vec();
            collect_mapper.insert(id, vec![(node, out_edges, in_edges)]);
        }

        for layer in &layers {
            for (id, node) in layer.nodes().into_iter().enumerate() {
                let out_edges = layer.out_edges_raw(node).collect_vec();
                let in_edges = layer.in_edges_raw(node).collect_vec();
                collect_mapper.entry(id).and_modify(|d| {
                    d.push((node, out_edges, in_edges));
                });
            }
        }

        unimplemented!()


        // for base_node in base.nodes() {
        //     let start_node_index = highway_nodes.len();
        //     let start_edge_index = edges.len();

        //     // let node_edges = base.out_edges_raw(node).map(|(id, e)| {
        //     //     NetworkEdge {
        //     //         level: 0,

        //     //     }
        //     // });

        //     for layer in layers {
        //         for n in layer.nodes() {
                    
        //         }
        //     }

        //     nodes.push(HighwayNetworkBaseNode {
        //         start_node_index,
        //         start_edge_index,
        //         end_node_index: unimplemented!(),
        //         last_edge_index: unimplemented!(),
        //     });
        // }
        // Self {
        //     max_level,
        //     nodes,
        //     highway_nodes,
        //     edges,
        // }
    }

    pub fn node(&self, node: NodeId) -> &HighwayNetworkBaseNode {
        &self.nodes[node.0 as usize]
    }

    pub fn node_level(&self, node: NodeId, level: u8) -> Option<&HighwayNetworkLevelNode> {
        let node = self.node(node);
        let levels = node.end_node_index - node.start_node_index;

        if level as u32 > levels {
            return None;
        }

        self.highway_nodes
            .get(node.start_node_index as usize + level as usize)
    }

    pub fn direction_edges(
        &self,
        node: NodeId,
        level: u8,
        direction: EdgeDirection,
    ) -> Option<EdgeIterator> {
        EdgeIterator::new(node, level, direction, self)
    }

    pub fn out_edges(&self, node: NodeId, level: u8) -> Option<EdgeIterator> {
        self.direction_edges(node, level, EdgeDirection::Forward)
    }

    pub fn in_edges(&self, node: NodeId, level: u8) -> Option<EdgeIterator> {
        self.direction_edges(node, level, EdgeDirection::Backward)
    }
}
