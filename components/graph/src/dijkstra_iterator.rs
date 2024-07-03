use crate::{
    Backward, DirectedNetworkGraph, EdgeId, Forward, NetworkData, NetworkEdge, NodeId, F32,
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
};

pub struct DijkstraIterator<'a, T: DijkstraDirection, D: NetworkData> {
    pub network: &'a DirectedNetworkGraph<D>,
    pub distance: f32,
    pub visited: HashSet<NodeId>,
    pub heap: BinaryHeap<Reverse<(F32, NodeId)>>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: DijkstraDirection, D: NetworkData> DijkstraIterator<'a, T, D> {
    pub fn new(network: &'a DirectedNetworkGraph<D>, start: NodeId) -> Self {
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

impl<'a, T, D> Iterator for DijkstraIterator<'a, T, D>
where
    T: DijkstraDirection,
    D: NetworkData,
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
    fn edges<'a, D: NetworkData>(
        network: &'a DirectedNetworkGraph<D>,
        node: NodeId,
    ) -> impl Iterator<Item = (EdgeId, &'a NetworkEdge)>;
}

impl DijkstraDirection for Forward {
    fn edges<'a, D: NetworkData>(
        network: &'a DirectedNetworkGraph<D>,
        node: NodeId,
    ) -> impl Iterator<Item = (EdgeId, &'a NetworkEdge)> {
        network.out_edges(node)
    }
}

impl DijkstraDirection for Backward {
    fn edges<'a, D: NetworkData>(
        network: &'a DirectedNetworkGraph<D>,
        node: NodeId,
    ) -> impl Iterator<Item = (EdgeId, &'a NetworkEdge)> {
        network.in_edges(node)
    }
}
