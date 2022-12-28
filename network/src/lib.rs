#![feature(map_try_insert)]
#![feature(is_sorted)]

pub mod iterators;
pub mod neighbourhood;

use iterators::Distanceable;
use itertools::Itertools;
pub use neighbourhood::*;
use petgraph::stable_graph::EdgeIndex;
use petgraph::stable_graph::IndexType;
use petgraph::stable_graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::EdgeRef;
use serde::Deserialize;
use serde::Serialize;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct HighwayIndex(usize);

pub type HighwayGraph<N, E> = StableDiGraph<N, E, HighwayIndex>;
pub type HighwayNodeIndex = NodeIndex<HighwayIndex>;
pub type HighwayEdgeIndex = EdgeIndex<HighwayIndex>;

unsafe impl IndexType for HighwayIndex {
    fn new(x: usize) -> Self {
        Self(x)
    }

    fn index(&self) -> usize {
        self.0
    }

    fn max() -> Self {
        HighwayIndex(usize::MAX)
    }
}

#[cfg(test)]
pub(crate) mod tests;

pub trait BypassNode {
    /// Bypass a node, and return nodes that have an edge removed
    fn bypass(&mut self, node: HighwayNodeIndex) -> Vec<HighwayNodeIndex>;
}

impl<N> BypassNode for HighwayGraph<N, Shorted> {
    fn bypass(&mut self, node: HighwayNodeIndex) -> Vec<HighwayNodeIndex> {
        let in_edges = self
            .edges_directed(node, petgraph::Direction::Incoming)
            .map(|e| {
                debug_assert_eq!(e.target(), node);
                (e.source(), e.weight().clone())
            })
            .collect_vec();

        let out_edges = self
            .edges_directed(node, petgraph::Direction::Outgoing)
            .map(|e| {
                debug_assert_eq!(e.source(), node);
                (e.target(), e.weight().clone())
            })
            .collect_vec();

        // The node has no receiving edges. Only outgoing. Then remove the node.
        if in_edges.len() == 0 {
            self.remove_node(node);
            return vec![];
        }

        // The node has no outgoing edges. Only incoming. Then remove the node.
        if out_edges.len() == 0 {
            self.remove_node(node);
            return vec![];
        }

        let mut touched = Vec::new();

        for (source, source_shorted) in in_edges {
            for (target, target_shorted) in &out_edges {
                if &source == target {
                    continue;
                }
                // Connect source to target.
                let combined_distance = source_shorted.distance() + target_shorted.distance();
                let mut skipped_nodes = vec![node];
                skipped_nodes.extend(source_shorted.skipped_nodes.clone());
                skipped_nodes.extend(target_shorted.skipped_nodes.clone());

                let mut skipped_edges = Vec::with_capacity(
                    source_shorted.skipped_edges.len() + target_shorted.skipped_edges.len(),
                );
                skipped_edges.extend(source_shorted.skipped_edges.clone());
                skipped_edges.extend(target_shorted.skipped_edges.clone());

                self.add_edge(
                    source,
                    *target,
                    Shorted {
                        distance: combined_distance,
                        skipped_nodes,
                        skipped_edges,
                    },
                );
                touched.push(*target);
            }
            touched.push(source);
        }

        touched
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shorted {
    pub distance: f32,
    /// Points to nodes in the previous layer
    pub skipped_nodes: Vec<HighwayNodeIndex>,
    /// Points to edges in the previous layer
    pub skipped_edges: Vec<HighwayEdgeIndex>,
}

impl Distanceable for Shorted {
    fn distance(&self) -> f32 {
        self.distance
    }
}
