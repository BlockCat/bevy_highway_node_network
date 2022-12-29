use crate::{HighwayGraph, HighwayNodeIndex};
use petgraph::Direction::Incoming;
use petgraph::visit::*;
use petgraph::{visit::EdgeRef, Direction::Outgoing};
use std::collections::{BinaryHeap, HashSet};

pub trait Distanceable {
    fn distance(&self) -> f32;
}

impl<E: Into<f32> + Copy> Distanceable for E {
    fn distance(&self) -> f32 {
        (*self).into()
    }
}

#[derive(Debug, PartialEq)]
struct IteratorHeapEntry {
    distance: f32,
    node: HighwayNodeIndex,
}

impl Eq for IteratorHeapEntry {}
impl Ord for IteratorHeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialOrd for IteratorHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.distance.partial_cmp(&other.distance) {
            Some(core::cmp::Ordering::Equal) => {}
            Some(ord) => return Some(ord.reverse()),
            _ => unreachable!(
                "Tried to compare: {} .comp({})",
                self.distance, other.distance
            ),
        }
        self.node.partial_cmp(&other.node)
    }
}

pub trait DijkstraIterator<N, E> {
    fn forward_iterator(&self, node: HighwayNodeIndex) -> ForwardDijkstraIterator<'_, N, E>;
    fn backward_iterator(&self, node: HighwayNodeIndex) -> BackwardDijkstraIterator<'_, N, E>;
}

impl<N, E> DijkstraIterator<N, E> for HighwayGraph<N, E> {
    fn forward_iterator(&self, node: HighwayNodeIndex) -> ForwardDijkstraIterator<'_, N, E> {
        let mut heap = BinaryHeap::new();
        heap.push(IteratorHeapEntry {
            distance: 0.0,
            node,
        });
        ForwardDijkstraIterator {
            network: self,
            distance: 0.0,
            visited: HashSet::new(),
            heap,
        }
    }

    fn backward_iterator(&self, node: HighwayNodeIndex) -> BackwardDijkstraIterator<'_, N, E> {
        let mut heap = BinaryHeap::new();
        heap.push(IteratorHeapEntry {
            distance: 0.0,
            node,
        });
        BackwardDijkstraIterator {
            network: self,
            distance: 0.0,
            visited: HashSet::new(),
            heap,
        }
    }
}

pub struct ForwardDijkstraIterator<'a, N, E> {
    pub distance: f32,
    network: &'a HighwayGraph<N, E>,
    visited: HashSet<HighwayNodeIndex>,
    heap: BinaryHeap<IteratorHeapEntry>,
}

pub struct BackwardDijkstraIterator<'a, N, E> {
    pub distance: f32,
    network: &'a HighwayGraph<N, E>,
    visited: HashSet<HighwayNodeIndex>,
    heap: BinaryHeap<IteratorHeapEntry>,
}

impl<'a, N, E> Iterator for ForwardDijkstraIterator<'a, N, E>
where
    E: Distanceable,
{
    type Item = (HighwayNodeIndex, f32);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(IteratorHeapEntry { distance, node }) = self.heap.pop() {
            if !self.visited.insert(node) {
                continue;
            }
            for edge in self.network.edges_directed(node, Outgoing) {
                let target = edge.target();
                let edge_distance = edge.weight().distance();

                let next_distance = distance + edge_distance;

                self.heap.push(IteratorHeapEntry {
                    node: target,
                    distance: next_distance,
                })
            }
            self.distance = distance;

            return Some((node, distance));
        }
        None
    }
}

impl<'a, N, E> Iterator for BackwardDijkstraIterator<'a, N, E>
where
    E: Distanceable,
{
    type Item = (HighwayNodeIndex, f32);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(IteratorHeapEntry { distance, node }) = self.heap.pop() {
            if !self.visited.insert(node) {
                continue;
            }
            for edge in self.network.edges_directed(node, Incoming) {
                let target = edge.source();
                let edge_distance = edge.weight().distance();

                let next_distance = distance + edge_distance;

                self.heap.push(IteratorHeapEntry {
                    node: target,
                    distance: next_distance,
                })
            }
            self.distance = distance;

            return Some((node, distance));
        }
        None
    }
}
