#![feature(map_try_insert)]
#![feature(is_sorted)]

pub mod iterators;
pub mod neighbourhood;
pub mod super_graph;
pub mod count_stable_graph;

use count_stable_graph::CountStableGraph;
use iterators::Distanceable;
use itertools::Itertools;
pub use neighbourhood::*;
use petgraph::stable_graph::EdgeIndex;
use petgraph::stable_graph::IndexType;
use petgraph::stable_graph::NodeIndex;
use petgraph::Direction;
use petgraph::visit::EdgeRef;
use serde::Deserialize;
use serde::Serialize;
use super_graph::SuperGraph;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct HighwayIndex(u32);

pub type HighwayGraph<N, E> = SuperGraph<N, E, HighwayIndex>;
pub type IntermediateGraph<N, E> = CountStableGraph<N, E, HighwayIndex>;
pub type HighwayNodeIndex = NodeIndex<HighwayIndex>;
pub type HighwayEdgeIndex = EdgeIndex<HighwayIndex>;

unsafe impl IndexType for HighwayIndex {
    fn new(x: usize) -> Self {
        Self(x as u32)
    }

    fn index(&self) -> usize {
        self.0 as usize
    }

    fn max() -> Self {
        HighwayIndex(u32::MAX)
    }
}

#[cfg(test)]
pub(crate) mod tests;

pub trait BypassNode {
    /// Bypass a node, and return nodes that have an edge removed
    fn bypass(&mut self, node: HighwayNodeIndex) -> Vec<HighwayNodeIndex>;
}

impl<N> BypassNode for IntermediateGraph<N, Shorted> {
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
        if self.edges_directed(node, Direction::Outgoing).count() == 0 {
            self.remove_node(node);
            return vec![];
        }

        // The node has no outgoing edges. Only incoming. Then remove the node.
        if self.edges_directed(node, Direction::Incoming).count() == 0 {
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
    /// Points to edges in the previous layer
    pub skipped_edges: Vec<HighwayEdgeIndex>,
}

impl Distanceable for Shorted {
    fn distance(&self) -> f32 {
        self.distance
    }
}
