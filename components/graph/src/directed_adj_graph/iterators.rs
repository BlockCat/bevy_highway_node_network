use crate::{EdgeDirection, EdgeId, NetworkEdge};
use std::{ops::Range, slice::Iter};

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
            .find(|(_, edge)| {
                self.direction == edge.direction || edge.direction == EdgeDirection::Both
            })?;

        Some((id.into(), edge))
    }
}
