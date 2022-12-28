use itertools::Itertools;
use network::{iterators::Distanceable, HighwayEdgeIndex, HighwayGraph, HighwayNodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

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

pub(crate) fn core_network_with_patch<N: Clone, E: Distanceable>(
    old_network: HighwayGraph<N, E>,
    contraction_factor: f32,
) -> HighwayGraph<N, Shorted> {
    let nodes = old_network.node_indices();
    let mut queue = HashNodeQueue::<HighwayNodeIndex>::from_iter(nodes);

    let mut next_network = old_network.map(
        |_, n| n.clone(),
        |id, e| Shorted {
            distance: e.distance(),
            skipped_nodes: vec![],
            skipped_edges: vec![id],
        },
    );

    while let Some(node) = queue.pop_front() {
        let out_edges = old_network
            .edges_directed(node, petgraph::Direction::Outgoing)
            .count() as f32;
        let in_edges = old_network
            .edges_directed(node, petgraph::Direction::Outgoing)
            .count() as f32;

        let short_cuts = out_edges * in_edges;
        let contraction = (out_edges + in_edges) * contraction_factor;

        if short_cuts < contraction {
            // Remove from the node from the new network.
            next_network.remove_node(node);
            let out_edges = old_network
                .edges_directed(node, petgraph::Direction::Outgoing)
                .collect_vec();
            let in_edges = old_network
                .edges_directed(node, petgraph::Direction::Incoming)
                .collect_vec();

            panic!("Need to be remove from next network. And Shorted can be combined, which is not the case right now");

            for source_edge in in_edges {
                let source = source_edge.source();
                let source_edge_id = source_edge.id();
                let source_edge = source_edge.weight();

                for target_edge in &out_edges {
                    let target = target_edge.target();
                    let target_edge_id = target_edge.id();
                    let target_edge = target_edge.weight();
                    // Connect source to target.
                    let combined_distance = source_edge.distance() + target_edge.distance();
                    next_network.add_edge(
                        source,
                        target,
                        Shorted {
                            distance: combined_distance,
                            skipped_nodes: vec![node],
                            skipped_edges: vec![source_edge_id, target_edge_id],
                        },
                    );

                    queue.push_back(target);
                }
                queue.push_back(source);
            }
        }
    }

    next_network
}

#[derive(Debug, Default, Clone)]
struct HashNodeQueue<T: Hash + Eq> {
    queue: VecDeque<T>,
    seen: HashSet<T>,
}

impl<T: Hash + Eq + Copy> HashNodeQueue<T> {
    fn from_iter<I: Iterator<Item = T>>(items: I) -> Self {
        let queue = VecDeque::from_iter(items);
        let seen = HashSet::from_iter(queue.iter().cloned());

        HashNodeQueue { queue, seen }
    }

    fn contains(&self, value: &T) -> bool {
        self.seen.contains(value)
    }

    fn pop_front(&mut self) -> Option<T> {
        let value = self.queue.pop_front()?;
        self.seen.remove(&value);
        Some(value)
    }

    fn push_back(&mut self, value: T) {
        if self.contains(&value) {
            self.queue.push_back(value);
        }
    }
}
