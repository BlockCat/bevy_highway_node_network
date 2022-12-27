use std::collections::BinaryHeap;

use network::{builder::EdgeDirection, highway_network::HighwayNetwork, NodeId};

pub fn shortest_path(source: NodeId, target: NodeId, network: &HighwayNetwork) {
    let mut forward = HighwayDijkstraIterator::default();
    let mut backward = HighwayDijkstraIterator::default();

    forward.push(
        source,
        0.0,
        0,
        network.node_level(source, 0).unwrap().forward_radius,
    );
    backward.push(
        target,
        0.0,
        0,
        network.node_level(target, 0).unwrap().backward_radius,
    );

    let mut forward_top_layer = Vec::new();
    let mut backward_top_layer = Vec::new();

    while !forward.is_empty() && !backward.is_empty() {
        rayon::join(
            || {
                handle_dijkstra_iteration(
                    EdgeDirection::Forward,
                    network,
                    &mut forward,
                    &mut forward_top_layer,
                )
            },
            || {
                handle_dijkstra_iteration(
                    EdgeDirection::Backward,
                    network,
                    &mut backward,
                    &mut backward_top_layer,
                )
            },
        );
    }
}

fn handle_dijkstra_iteration(
    direction: EdgeDirection,
    network: &HighwayNetwork,
    iterator: &mut HighwayDijkstraIterator,
    top_layer: &mut Vec<HeapEntry>,
) {
    let max_level = network.max_level;
    if let Some(entry) = iterator.pop() {
        let gap = if entry.gap.is_finite() {
            entry.gap
        } else {
            retrieve_radius(entry.node, entry.level, direction, network)
        };

        if gap.is_finite() && entry.level == max_level {
            top_layer.push(entry);
        }

        todo!("Settled from both dirs?");

        if let Some(edges) = network.direction_edges(entry.node, entry.level, direction) {
            for (_, edge) in edges {
                let mut level = entry.level;
                let mut gap = gap;
                while edge.distance() > gap {
                    level += 1;
                    gap = retrieve_radius(entry.node, level, direction, network);
                }

                if edge.level() < level {
                    continue;
                }

                if network
                    .node_level(entry.node, level)
                    .map(|x| !x.is_bypassed)
                    .unwrap_or(false)
                    && network
                        .node_level(edge.target(), level)
                        .unwrap()
                        .is_bypassed
                {
                    continue;
                }
                if level == max_level && level > entry.level {
                    top_layer.push(HeapEntry {
                        node: edge.target(),
                        distance: entry.distance + edge.distance(),
                        level,
                        gap: gap - edge.distance(),
                    });
                    continue;
                }

                iterator.push(
                    edge.target(),
                    entry.distance + edge.distance(),
                    level,
                    gap - edge.distance(),
                );
            }
        }
    }
}

fn retrieve_radius(
    node: NodeId,
    level: u8,
    direction: EdgeDirection,
    network: &HighwayNetwork,
) -> f32 {
    match direction {
        EdgeDirection::Forward => network
            .node_level(node, level)
            .filter(|node| !node.is_bypassed)
            .map(|node| node.forward_radius)
            .unwrap_or(std::f32::INFINITY),
        EdgeDirection::Backward => network
            .node_level(node, level)
            .filter(|node| !node.is_bypassed)
            .map(|node| node.backward_radius)
            .unwrap_or(std::f32::INFINITY),
        _ => unreachable!(),
    }
}

#[derive(Default)]
struct HighwayDijkstraIterator {
    queue: BinaryHeap<HeapEntry>,
}

impl HighwayDijkstraIterator {
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn pop(&mut self) -> Option<HeapEntry> {
        self.queue.pop()
    }

    pub fn push(&mut self, node: NodeId, distance: f32, level: u8, gap: f32) {
        self.queue.push(HeapEntry {
            node,
            distance,
            level,
            gap,
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
struct HeapEntry {
    node: NodeId,
    distance: f32,
    level: u8,
    gap: f32,
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match self.node.partial_cmp(&other.node) {
            Some(Ordering::Equal) => {}
            Some(ord) => return Some(ord.reverse()),
            None => return None,
        }
        match self.level.partial_cmp(&other.level) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.distance.partial_cmp(&other.distance) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        self.gap.partial_cmp(&other.gap).map(|x| x.reverse())
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
