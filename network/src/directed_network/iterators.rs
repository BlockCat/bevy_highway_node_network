use crate::{
    builder::EdgeDirection, DirectedNetworkGraph, EdgeId, NetworkData, NetworkEdge, NodeId,
};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
    ops::Range,
    slice::Iter,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct F32(pub f32);

impl Ord for F32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for F32 {}

pub struct EdgeIterator<'a> {
    range: Range<u32>,
    edges: Iter<'a, NetworkEdge>,
    direction: EdgeDirection,
}

impl<'a> EdgeIterator<'a> {
    pub fn new(range: Range<u32>, edges: Iter<'a, NetworkEdge>, direction: EdgeDirection) -> Self {
        Self {
            range,
            edges,
            direction,
        }
    }
}

impl<'a> Iterator for EdgeIterator<'a> {
    type Item = (EdgeId, &'a NetworkEdge);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, edge) = self
            .range
            .by_ref()
            .zip(self.edges.by_ref())
            .find(|(id, edge)| {
                self.direction == edge.direction || edge.direction == EdgeDirection::Both
            })?;

        Some((id.into(), edge))
    }
}

pub struct ForwardDijkstraIterator<'a, D: NetworkData> {
    pub network: &'a DirectedNetworkGraph<D>,
    pub distance: f32,
    pub visited: HashSet<NodeId>,
    pub heap: BinaryHeap<Reverse<(F32, NodeId)>>,
}

pub struct BackwardDijkstraIterator<'a, D: NetworkData> {
    pub distance: f32,
    pub network: &'a DirectedNetworkGraph<D>,
    pub visited: HashSet<NodeId>,
    pub heap: BinaryHeap<Reverse<(F32, NodeId)>>,
}

impl<'a, D: NetworkData> Iterator for ForwardDijkstraIterator<'a, D> {
    type Item = (NodeId, f32);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Reverse((F32(distance), node))) = self.heap.pop() {
            if !self.visited.insert(node) {
                continue;
            }
            for (_, edge) in self.network.out_edges(node) {
                let target = edge.target();
                let edge_distance = edge.distance();

                self.heap
                    .push(Reverse((F32(distance + edge_distance), target)));
            }

            self.distance = distance;

            return Some((node, distance));
        }
        return None;
    }
}

impl<'a, D: NetworkData> Iterator for BackwardDijkstraIterator<'a, D> {
    type Item = (NodeId, f32);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(Reverse((F32(distance), node))) = self.heap.pop() {
            if !self.visited.insert(node) {
                continue;
            }
            for (_, edge) in self.network.in_edges(node) {
                let source = edge.target();
                let edge_distance = edge.distance();

                self.heap
                    .push(Reverse((F32(distance + edge_distance), source)));
            }

            self.distance = distance;

            return Some((node, distance));
        }
        return None;
    }
}
