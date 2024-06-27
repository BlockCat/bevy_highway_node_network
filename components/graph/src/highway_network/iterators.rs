use super::{HighwayNetwork, NetworkEdge};
use crate::{builder::EdgeDirection, EdgeId, NodeId};
use core::slice::Iter;
use std::ops::Range;

pub struct EdgeIterator<'a> {
    network: &'a HighwayNetwork,
    level: u8,
    range: Range<u32>,
    edges: Iter<'a, NetworkEdge>,
    direction: EdgeDirection,
}

impl<'a> EdgeIterator<'a> {
    pub fn new(
        node: NodeId,
        level: u8,
        direction: EdgeDirection,
        network: &'a HighwayNetwork,
    ) -> Option<Self> {
        let layer_node = network.node_level(node, level)?;
        let range = layer_node.start_edge_index..layer_node.last_edge_index;
        let edges = network.edges
            [layer_node.start_edge_index as usize..layer_node.last_edge_index as usize]
            .iter();
        Some(Self {
            network,
            level,
            range,
            edges,
            direction,
        })
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
                (self.direction == edge.direction || edge.direction == EdgeDirection::Both)
                    && self
                        .network
                        .node_level(edge.target_node, self.level)
                        .is_some()
            })?;

        Some((id.into(), edge))
    }
}
