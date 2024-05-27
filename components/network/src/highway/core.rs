use super::intermediate_network::IntermediateNetwork;
use crate::NodeId;
use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

pub(crate) fn core_network_with_patch(
    mut intermediate_network: IntermediateNetwork,
    contraction_factor: f32,
) -> IntermediateNetwork {
    let nodes = {
        let mut n = intermediate_network.nodes();
        n.sort();
        n
    };

    let mut queue = HashNodeQueue::<NodeId>::from_vec(&nodes);

    while let Some(node) = queue.pop_front() {
        let out_edges = &intermediate_network
            .out_edges(node)
            .map(|x| x.len() as f32)
            .unwrap_or_default();
        let in_edges = &intermediate_network
            .in_edges(node)
            .map(|x| x.len() as f32)
            .unwrap_or_default();

        let short_cuts = (out_edges * in_edges) as f32;
        let contraction = (out_edges + in_edges) * contraction_factor;

        if short_cuts < contraction {
            let mut touched = intermediate_network.bypass(node);
            touched.sort();
            for touched in touched {
                queue.push_back(touched);
            }
        }
    }

    intermediate_network
}

#[derive(Debug, Default, Clone)]
struct HashNodeQueue<T: Hash + Eq> {
    queue: VecDeque<T>,
    seen: HashSet<T>,
}

impl<T: Hash + Eq + Copy> HashNodeQueue<T> {
    fn from_vec(items: &[T]) -> Self {
        let queue = VecDeque::from_iter(items.iter().cloned());
        let mut seen = HashSet::with_capacity(items.len());
        seen.extend(items.iter().cloned());

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
