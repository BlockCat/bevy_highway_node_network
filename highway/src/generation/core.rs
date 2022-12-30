use network::{iterators::Distanceable, HighwayNodeIndex};
use network::{BypassNode, IntermediateGraph, Shorted};
use petgraph::visit::IntoNodeIdentifiers;

use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

pub(crate) fn core_network_with_patch<N: Clone, E: Distanceable>(
    old_network: IntermediateGraph<N, E>,
    contraction_factor: f32,
) -> IntermediateGraph<N, Shorted> {
    let nodes = old_network.node_identifiers();
    let mut queue = HashNodeQueue::<HighwayNodeIndex>::from_iter(nodes);

    let mut next_network = old_network.map(
        |_, n| n.clone(),
        |id, e| Shorted {
            distance: e.distance(),
            skipped_edges: vec![id],
        },
    );

    drop(old_network);

    while let Some(node) = queue.pop_front() {
        let out_edges = next_network.edge_count_out(node) as f32;
        let in_edges = next_network.edge_count_in(node) as f32;

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
        if self.seen.insert(value) {
            self.queue.push_back(value);
        }
    }
}
