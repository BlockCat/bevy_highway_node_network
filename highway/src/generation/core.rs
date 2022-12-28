use itertools::Itertools;
use network::{BypassNode, Shorted};
use network::{iterators::Distanceable, HighwayEdgeIndex, HighwayGraph, HighwayNodeIndex};
use petgraph::{
    visit::{EdgeRef, IntoEdgesDirected},
    Direction,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};


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

    drop(old_network);

    while let Some(node) = queue.pop_front() {
        let out_edges = next_network
            .edges_directed(node, Direction::Outgoing)
            .count() as f32;
        let in_edges = next_network
            .edges_directed(node, Direction::Outgoing)
            .count() as f32;

        let short_cuts = out_edges * in_edges;
        let contraction = (out_edges + in_edges) * contraction_factor;

        if short_cuts <= contraction {
            let touched = next_network.bypass(node);
            for touched in touched {
                queue.push_back(touched);
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
        if !self.contains(&value) {
            self.queue.push_back(value);
        }
    }
}
