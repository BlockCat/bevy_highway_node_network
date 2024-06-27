use super::ComputedState;
use graph::{iterators::F32, DirectedNetworkGraph, EdgeId, NetworkData, NodeId};
use std::{
    cmp::Reverse,
    collections::{HashMap, VecDeque},
};

use super::dag::*;

pub fn calculate_edges<D: NetworkData>(
    s0: NodeId,
    computed: &ComputedState,
    network: &DirectedNetworkGraph<D>,
) -> Vec<(NodeId, EdgeId)> {
    let (sorted_order, dag) = create_directed_acyclic_graph(s0, computed, network);
    let edges = collect_next_level_edges(s0, sorted_order, dag, computed);

    edges
}

pub fn create_directed_acyclic_graph<D: NetworkData>(
    s0: NodeId,
    computed: &ComputedState,
    network: &DirectedNetworkGraph<D>,
) -> (
    VecDeque<(NodeId, (EdgeId, f32))>,
    HashMap<NodeId, VisitedState>,
) {
    let mut heap = HighwayNodeQueue::new(2000, 3000);

    let mut settled_order = VecDeque::with_capacity(3000);

    // println!("Continue growing the DAG, and stop when there are no more active nodes");

    initialize_heap(s0, network, &mut heap);

    while let Some(entry) = heap.pop() {
        let border_distance = border_distance(
            s0,
            entry.state.current,
            &entry.parents,
            computed,
            entry.border_distance,
        );

        let reference_distance = reference_distance(&entry, border_distance, &heap.visited);
        heap.visited(
            entry.state.current,
            VisitedState {
                distance: entry.state.distance,
                border_distance,
                reference_distance,
                parents: entry.parents,
            },
        );

        settled_order.push_front((
            entry.state.current,
            (entry.state.parent.parent_edge, entry.state.distance),
        ));

        let should_abort = (reference_distance + computed.backward.radius(entry.state.current))
            < entry.state.distance;

        let active = entry.parent_active && !should_abort;

        for (id, child_edge) in network.out_edges(entry.state.current) {
            let child = child_edge.target();
            let next_distance = entry.state.distance + child_edge.distance();

            heap.push(DijkstraNodeState {
                current: child,
                distance: next_distance,
                parent: ParentEntry {
                    parent: entry.state.current,
                    parent_edge_distance: child_edge.distance(),
                    parent_edge: id.into(),
                    active,
                },
            });
        }
    }

    (settled_order, heap.visited)
}

fn collect_next_level_edges(
    s0: NodeId,
    mut sorted_nodes: VecDeque<(NodeId, (EdgeId, f32))>,
    nodes: HashMap<NodeId, VisitedState>,
    computed: &ComputedState,
) -> Vec<(NodeId, EdgeId)> {
    let mut collected_edges = Vec::new();
    let mut tentative_slacks = HashMap::new();

    debug_assert!(sorted_nodes
        .iter()
        .is_sorted_by_key(|x| Reverse(F32(x.1 .1))));

    while let Some((node, (_, distance))) = sorted_nodes.pop_front() {
        if distance < computed.forward.radius(s0) {
            // return collected_edges;
            continue;
        }

        let slack = *tentative_slacks
            .entry(node)
            .or_insert_with(|| computed.backward.radius(node));

        for (parent, (edge_id, distance)) in &nodes[&node].parents {
            let slack_parent = slack - distance;

            if slack_parent < 0.0 {
                collected_edges.push((*parent, edge_id.unwrap()));
            }

            let tentative_slack_parent = tentative_slacks
                .entry(*parent)
                .or_insert_with(|| computed.backward.radius(*parent));

            *tentative_slack_parent = f32::min(*tentative_slack_parent, slack_parent);
        }
    }

    collected_edges
}

fn initialize_heap<D: NetworkData>(
    s0: NodeId,
    network: &DirectedNetworkGraph<D>,
    heap: &mut HighwayNodeQueue,
) {
    heap.visited(
        s0,
        VisitedState {
            border_distance: 0.0,
            reference_distance: f32::INFINITY,
            distance: 0.0,
            parents: HashMap::from([(s0, (None, 0.0))]),
        },
    );
    for (id, edge) in network.out_edges(s0) {
        // assert!(s0 != edge.target());
        heap.push(DijkstraNodeState {
            distance: edge.distance(),
            current: edge.target(),
            parent: ParentEntry {
                parent: s0,
                parent_edge_distance: edge.distance(),
                parent_edge: id.into(),
                active: true,
            },
        });
    }
}

fn border_distance<A>(
    s0: NodeId,
    node: NodeId,
    parents: &HashMap<NodeId, (A, f32)>,
    computed: &ComputedState,
    parent_border_distance: f32,
) -> f32 {
    if let Some((_, distance)) = parents.get(&s0) {
        *distance + computed.forward.radius(node)
    } else {
        parent_border_distance
    }
}

fn reference_distance(
    entry: &HighwayQueueEntry,
    border_distance: f32,
    visited: &HashMap<NodeId, VisitedState>,
) -> f32 {
    let distance = entry.state.distance;
    let reference_distance = entry.reference_distance;
    let parents = &entry.parents;
    if reference_distance == f32::INFINITY && distance > border_distance {
        parents
            .keys()
            .flat_map(|parent| visited[parent].parents.iter())
            .map(|sp| visited[sp.0].distance)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    } else {
        reference_distance
    }
}

#[cfg(test)]
mod tests {

    use graph::{
        builder::EdgeDirection, create_network, DirectedNetworkGraph, NetworkEdge, NetworkNode,
        NodeId,
    };
    use std::collections::HashSet;

    use crate::generation::dijkstra::collect_next_level_edges;

    use super::{create_directed_acyclic_graph, ComputedState};
    // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
    pub fn create_ref_network_1() -> DirectedNetworkGraph<()> {
        let nodes = vec![
            NetworkNode::new(0, 0, 2),
            NetworkNode::new(1, 2, 5),
            NetworkNode::new(2, 5, 7),
            NetworkNode::new(3, 7, 10),
            NetworkNode::new(4, 10, 13),
            NetworkNode::new(5, 13, 16),
        ];
        let edges = vec![
            NetworkEdge::new(0, 1u32.into(), 10.0, EdgeDirection::Forward), // A -> B
            NetworkEdge::new(1, 2u32.into(), 15.0, EdgeDirection::Forward), // A -> C
            NetworkEdge::new(2, 3u32.into(), 12.0, EdgeDirection::Forward), // B -> D
            NetworkEdge::new(3, 5u32.into(), 15.0, EdgeDirection::Forward), // B -> F
            NetworkEdge::new(4, 0u32.into(), 10.0, EdgeDirection::Backward), // A <- B
            NetworkEdge::new(5, 4u32.into(), 10.0, EdgeDirection::Forward), // C -> E
            NetworkEdge::new(6, 0u32.into(), 15.0, EdgeDirection::Backward), // A <- C
            NetworkEdge::new(7, 4u32.into(), 2.0, EdgeDirection::Forward),  // D -> E
            NetworkEdge::new(8, 5u32.into(), 1.0, EdgeDirection::Forward),  // D -> F
            NetworkEdge::new(9, 1u32.into(), 12.0, EdgeDirection::Backward), // B <- D
            NetworkEdge::new(10, 2u32.into(), 10.0, EdgeDirection::Backward), // C <- E
            NetworkEdge::new(11, 3u32.into(), 2.0, EdgeDirection::Backward), // D <- E
            NetworkEdge::new(12, 5u32.into(), 5.0, EdgeDirection::Backward), // F <- E
            NetworkEdge::new(13, 4u32.into(), 5.0, EdgeDirection::Forward), // F -> E
            NetworkEdge::new(14, 1u32.into(), 15.0, EdgeDirection::Backward), // B <- F
            NetworkEdge::new(15, 3u32.into(), 1.0, EdgeDirection::Backward), // D <- F
        ];

        DirectedNetworkGraph::new(nodes, edges, ())
    }

    pub fn create_undirected_network() -> DirectedNetworkGraph<()> {
        create_network!(
            0..16,
            0 => 1; 3.0,
            1 => 0; 3.0,
            1 => 2; 2.0,
            2 => 1; 2.0,
            2 => 3; 2.0,
            3 => 2; 2.0,
            3 => 0; 2.0,
            0 => 3; 2.0,

            7 => 6; 2.0,
            6 => 7; 2.0,
            6 => 5; 2.0,
            5 => 6; 2.0,
            4 => 5; 3.0,
            5 => 4; 3.0,
            4 => 7; 2.0,
            7 => 4; 2.0,

            8 => 9; 3.0,
            9 => 8; 3.0,
            9 => 10; 2.0,
            10 => 9; 2.0,
            10 => 11; 2.0,
            11 => 10; 2.0,
            11 => 8; 2.0,
            8 => 11; 2.0,

            12 => 13; 2.0,
            13 => 12; 2.0,
            13 => 14; 2.0,
            14 => 13; 2.0,
            14 => 15; 3.0,
            15 => 14; 3.0,
            15 => 12; 2.0,
            12 => 15; 2.0,

            12 => 16; 6.0,
            16 => 12; 6.0,
            16 => 0; 5.0,
            0 => 16; 5.0,

            13 => 7; 7.0,
            7 => 13; 7.0,

            2 => 4; 6.0,
            4 => 2; 6.0,

            14 => 11; 14.0,
            11 => 14; 14.0
        )
    }

    #[test]
    fn forward_test() {
        let network = create_ref_network_1();
        let computed = ComputedState::new(3, &network);

        create_directed_acyclic_graph(NodeId(0), &computed, &network);
    }

    #[test]
    fn level_test() {
        let network = create_undirected_network();
        let computed = super::ComputedState::new(4, &network);
        let s0 = NodeId(12);
        let edges = create_directed_acyclic_graph(s0, &computed, &network);

        println!("DAG:");
        for (n, i) in &edges.1 {
            println!("N: {},\t {:?}", n.0, i);
        }

        println!("Edges: {:?}", edges.0);

        let next_edges = collect_next_level_edges(s0, edges.0, edges.1, &computed);

        println!("Added:");
        for (parent, id) in next_edges {
            let edge = network.edge(id);
            assert_ne!(parent, edge.target());
            println!("ID: {:?} - {:?}", id, edge);
        }
    }

    #[test]
    fn test_all() {
        let network = create_undirected_network();
        let computed = super::ComputedState::new(4, &network);

        let mut next_edges = HashSet::new();

        for n in 0..=16 {
            let edges = super::calculate_edges(NodeId(n), &computed, &network);

            next_edges.extend(edges);
        }

        let mut next_edges = next_edges.into_iter().collect::<Vec<_>>();

        next_edges.sort();

        println!("Added:");
        for (_, id) in next_edges {
            let edge = network.edge(id);
            println!("ID: {:?} - {:?}", id, edge);
        }
    }
}
