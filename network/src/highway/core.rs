use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
};

use crate::{DirectedNetworkGraph, EdgeId, NetworkData, NodeId};

pub fn core_network<D: NetworkData>(
    network: &mut DirectedNetworkGraph<D>,
    contraction_factor: f32,
) {
    let mut core_network = CoreNetwork::from(network);
    let node_ids = (0..network.nodes().len()).map(|x| NodeId::from(x)).collect::<Vec<_>>();

    let mut queue = HashNodeQueue::<NodeId>::from_vec(&node_ids);

    while let Some(node) = queue.pop_front() {
        let out_edges = &core_network.out_edges[&node];
        let in_edges = &core_network.in_edges[&node];

        let short_cuts = (out_edges.len() * in_edges.len()) as f32;
        let contraction = (out_edges.len() + in_edges.len()) as f32 * contraction_factor;

        if short_cuts < contraction {
            let mut touched = core_network.bypass(node);
            touched.sort();
            for touched in touched {
                queue.push_back(touched);
            }
        }
    }
}

struct CoreNetwork {
    nodes: HashSet<NodeId>,
    edge_count: usize,
    out_edges: HashMap<NodeId, HashMap<NodeId, EdgeId>>,
    in_edges: HashMap<NodeId, HashMap<NodeId, EdgeId>>,
    
}

impl CoreNetwork {
    fn from<D: NetworkData>(network: &DirectedNetworkGraph<D>) -> Self {
        todo!()
    }

    fn bypass(&mut self, node: NodeId) -> Vec<NodeId> {
        let parents = &self.out_edges[&node];
        let children = &self.out_edges[&node];
        
        for parent in parents {
            for child in children {
                // self.out_edges.get_mut(&parent.0).unwrap().insert(k, v)
            }
        }

        parents.keys().chain(children.keys()).cloned().collect()
    }
}

struct HashNodeQueue<T: Hash + Eq> {
    queue: VecDeque<T>,
    seen: HashSet<T>,
}

impl<T: Hash + Eq + Copy> HashNodeQueue<T> {
    fn new() -> Self {
        Self {
            queue: Default::default(),
            seen: Default::default(),
        }
    }
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
        if self.seen.insert(value) {
            self.queue.push_back(value);
        }
    }
}
