use crate::{Backward, DirectedNetworkGraph, EdgeId, Forward, NetworkEdge, NodeId, F32};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
};

pub struct DijkstraIterator<'a, T: DijkstraDirection> {
    pub network: &'a DirectedNetworkGraph,
    pub distance: f32,
    pub visited: HashSet<NodeId>,
    pub heap: BinaryHeap<Reverse<(F32, NodeId)>>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: DijkstraDirection> DijkstraIterator<'a, T> {
    pub fn new(network: &'a DirectedNetworkGraph, start: NodeId) -> Self {
        let mut heap = BinaryHeap::new();
        heap.push(Reverse((F32(0.0), start)));

        Self {
            network,
            distance: 0.0,
            visited: HashSet::new(),
            heap,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for DijkstraIterator<'a, T>
where
    T: DijkstraDirection,
{
    type Item = (NodeId, f32);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Reverse((F32(distance), node))) = self.heap.pop() {
            if !self.visited.insert(node) {
                continue;
            }
            for (_, edge) in T::edges(self.network, node) {
                let target = edge.target();
                let edge_distance = edge.weight();

                self.heap
                    .push(Reverse((F32(distance + edge_distance), target)));
            }

            self.distance = distance;

            return Some((node, distance));
        }
        return None;
    }
}

pub trait DijkstraDirection {
    fn edges<'a>(
        network: &'a DirectedNetworkGraph,
        node: NodeId,
    ) -> impl Iterator<Item = (EdgeId, &'a NetworkEdge)>;
}

impl DijkstraDirection for Forward {
    fn edges<'a>(
        network: &'a DirectedNetworkGraph,
        node: NodeId,
    ) -> impl Iterator<Item = (EdgeId, &'a NetworkEdge)> {
        network.out_edges(node)
    }
}

impl DijkstraDirection for Backward {
    fn edges<'a>(
        network: &'a DirectedNetworkGraph,
        node: NodeId,
    ) -> impl Iterator<Item = (EdgeId, &'a NetworkEdge)> {
        network.in_edges(node)
    }
}
