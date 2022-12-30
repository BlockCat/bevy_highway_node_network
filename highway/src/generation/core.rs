use network::{iterators::Distanceable, HighwayNodeIndex};
use network::{BypassNode, IntermediateGraph, Shorted};
use petgraph::visit::IntoNodeIdentifiers;
use petgraph::Direction::{Incoming, Outgoing};
use petgraph::{algo, Graph};

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

    // drop(old_network);

    // next_network.recount();

    println!("Recounted");
    while let Some(node) = queue.pop_front() {
        if !next_network.contains_node(node) {
            continue;
        }

        let out_edges = next_network.edges_directed(node, Outgoing).count();
        let in_edges = next_network.edges_directed(node, Incoming).count();

        // debug_assert_eq!(
        //     next_network.edge_count_out(node),
        //     next_network.edges_directed(node, Outgoing).count()
        // );
        // debug_assert_eq!(
        //     next_network.edge_count_in(node),
        //     next_network.edges_directed(node, Incoming).count()
        // );

        let short_cuts = (out_edges * in_edges) as f32;
        let contraction = (out_edges + in_edges) as f32 * contraction_factor;

        if queue.queue.len() % 1000 == 0 {
            println!(
                "Si: {}, n: {}, e: {}, {} < {}",
                queue.queue.len(),
                next_network.node_count(),
                next_network.edge_count(),
                out_edges * in_edges,
                (out_edges + in_edges) as f32 * contraction_factor,
            );
        }

        if short_cuts <= contraction {
            for touched in next_network.bypass(node) {
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
