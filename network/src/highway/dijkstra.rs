use super::ComputedState;
use crate::{iterators::F32, DirectedNetworkGraph, EdgeId, NetworkData, NodeId};
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
        assert!(s0 != edge.target());
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

    use std::collections::HashSet;

    use crate::{
        highway::dijkstra::{collect_next_level_edges, create_directed_acyclic_graph},
        tests::{create_ref_network_1, create_undirected_network},
        NodeId,
    };

    use super::ComputedState;

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
