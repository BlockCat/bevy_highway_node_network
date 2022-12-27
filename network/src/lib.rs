#![feature(map_try_insert)]
#![feature(is_sorted)]

pub mod iterators;
pub mod neighbourhood;

pub use neighbourhood::*;
use petgraph::stable_graph::EdgeIndex;
use petgraph::stable_graph::IndexType;
use petgraph::stable_graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use serde::Deserialize;
use serde::Serialize;

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct HighwayIndex(usize);

pub type HighwayGraph<N, E> = StableDiGraph<N, E, HighwayIndex>;
pub type HighwayNodeIndex = NodeIndex<HighwayIndex>;
pub type HighwayEdgeIndex = EdgeIndex<HighwayIndex>;

unsafe impl IndexType for HighwayIndex {
    fn new(x: usize) -> Self {
        Self(x)
    }

    fn index(&self) -> usize {
        self.0
    }

    fn max() -> Self {
        HighwayIndex(usize::MAX)
    }
}

#[cfg(test)]
pub(crate) mod tests;
