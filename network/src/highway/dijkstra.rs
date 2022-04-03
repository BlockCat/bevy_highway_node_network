use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use crate::{DirectedNetworkGraph, EdgeId, NetworkData, NetworkEdge, NetworkNode, NodeId};

use super::ComputedState;

#[derive(Debug, PartialEq, PartialOrd)]
struct ParentEntry {
    parent: NodeId,
    parent_edge: EdgeId,
    parent_edge_distance: f32,
}

#[derive(Debug, PartialEq)]
struct DijkstraNodeState {
    distance: f32,
    current: NodeId,
    parent: ParentEntry,
}

impl PartialOrd for DijkstraNodeState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.distance.partial_cmp(&other.distance) {
            Some(core::cmp::Ordering::Equal) => {}
            Some(core::cmp::Ordering::Greater) => return Some(core::cmp::Ordering::Less),
            Some(core::cmp::Ordering::Less) => return Some(core::cmp::Ordering::Greater),
            None => return None,
        }

        match self.current.partial_cmp(&other.current) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }

        self.parent.partial_cmp(&other.parent)
    }
}

impl Ord for DijkstraNodeState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Eq for DijkstraNodeState {}

#[derive(Debug)]
struct VisitedState {
    border_distance: f32,
    reference_distance: f32,
    distance: f32,
    parents: HashMap<NodeId, (Option<EdgeId>, f32)>,
}

pub fn calculate_edges<D: NetworkData>(
    s0: NodeId,
    computed: &ComputedState,
    network: &DirectedNetworkGraph<D>,
) -> Vec<EdgeId> {
    let (sorted_order, dag) = create_directed_acyclic_graph(s0, computed, network);
    let edges = collect_next_level_edges(s0, sorted_order, dag, computed);

    edges
}

fn create_directed_acyclic_graph<D: NetworkData>(
    s0: NodeId,
    computed: &ComputedState,
    network: &DirectedNetworkGraph<D>,
) -> (
    VecDeque<(NodeId, (EdgeId, f32))>,
    HashMap<NodeId, VisitedState>,
) {
    let mut heap: BinaryHeap<DijkstraNodeState> = BinaryHeap::with_capacity(2000);

    let mut visited: HashMap<NodeId, VisitedState> = HashMap::with_capacity(3000);

    let mut settled_order = VecDeque::with_capacity(3000);

    // println!("Continue growing the DAG, and stop when there are no more active nodes");

    initialize_heap(s0, network, &mut heap, &mut visited);

    while let Some(mut state) = heap.pop() {
        if visited.contains_key(&state.current) {
            debug_assert!(state.distance > visited[&state.current].distance);
            continue;
        }

        let (parent_border_distance, parent_reference_distance, parents) =
            caculate_distances(&visited, &mut state, &mut heap);

        let border_distance = border_distance(
            s0,
            state.current,
            state.distance,
            &parents,
            computed,
            parent_border_distance,
        );

        let reference_distance = reference_distance(
            state.distance,
            border_distance,
            parent_reference_distance,
            &parents,
            &visited,
        );

        visited.insert(
            state.current,
            VisitedState {
                distance: state.distance,
                border_distance: border_distance,
                reference_distance,
                parents,
            },
        );

        settled_order.push_front((state.current, (state.parent.parent_edge, state.distance)));

        let should_abort =
            (reference_distance + computed.backward.radius(state.current)) < state.distance;

        if !should_abort {
            for (child_edge, id) in network.out_edges(state.current) {
                let child = child_edge.target();
                let next_distance = state.distance + child_edge.distance();

                heap.push(DijkstraNodeState {
                    current: child,
                    distance: next_distance,
                    parent: ParentEntry {
                        parent: state.current,
                        parent_edge_distance: child_edge.distance(),
                        parent_edge: id.into(),
                    },
                });
            }
            // } else {
            // println!(
            //     "Aborted node: {}: {} < {}, a(x)={}, rb(x)={}",
            //     *state.current,
            //     reference_distance + computed.backward.radius[*state.current],
            //     state.distance,
            //     reference_distance,
            //     computed.backward.radius[*state.current]
            // );
        }
    }

    (settled_order, visited)
}

fn collect_next_level_edges(
    s0: NodeId,
    mut sorted_nodes: VecDeque<(NodeId, (EdgeId, f32))>,
    nodes: HashMap<NodeId, VisitedState>,
    computed: &ComputedState,
) -> Vec<EdgeId> {
    let mut collected_edges = Vec::new();
    let mut tentative_slacks = HashMap::new();

    while let Some((node, (edge, distance))) = sorted_nodes.pop_front() {
        if distance <= computed.forward.radius(s0) {
            return collected_edges;
        }

        let slack = *tentative_slacks
            .entry(node)
            .or_insert_with(|| computed.backward.radius(node));

        for (parent, (edge_id, distance)) in &nodes[&node].parents {
            let slack_parent = slack - distance;

            if slack_parent < 0.0 {
                collected_edges.push(edge_id.unwrap());
            }

            let tentative_slack_parent = tentative_slacks
                .entry(*parent)
                .or_insert_with(|| computed.backward.radius(*parent));

            *tentative_slack_parent = f32::min(*tentative_slack_parent, slack_parent);
        }
    }

    collected_edges
}

fn caculate_distances(
    visited: &HashMap<NodeId, VisitedState>,
    state: &mut DijkstraNodeState,
    heap: &mut BinaryHeap<DijkstraNodeState>,
) -> (f32, f32, HashMap<NodeId, (Option<EdgeId>, f32)>) {
    let mut parent_border_distance = visited[&state.parent.parent].border_distance;
    let mut parent_reference_distance = visited[&state.parent.parent].reference_distance;
    let mut parents = HashMap::from([(
        state.parent.parent,
        (
            Some(state.parent.parent_edge),
            state.parent.parent_edge_distance,
        ),
    )]);
    while let Some(peek) = heap
        .peek()
        .filter(|next| next.current == state.current && next.distance == state.distance)
    {
        let parent_visited = &visited[&peek.parent.parent];
        parent_border_distance = f32::max(parent_border_distance, parent_visited.border_distance);

        parent_reference_distance =
            f32::max(parent_reference_distance, parent_visited.reference_distance);

        parents.insert(
            peek.parent.parent,
            (
                Some(peek.parent.parent_edge),
                peek.parent.parent_edge_distance,
            ),
        );
        *state = heap.pop().unwrap();
    }
    (parent_border_distance, parent_reference_distance, parents)
}

fn initialize_heap<D: NetworkData>(
    s0: NodeId,
    network: &DirectedNetworkGraph<D>,
    heap: &mut BinaryHeap<DijkstraNodeState>,
    visited: &mut HashMap<NodeId, VisitedState>,
) {
    visited.insert(
        s0,
        VisitedState {
            border_distance: 0.0,
            reference_distance: f32::INFINITY,
            distance: 0.0,
            parents: HashMap::from([(s0, (None, 0.0))]),
        },
    );
    for (edge, id) in network.out_edges(s0) {
        heap.push(DijkstraNodeState {
            distance: edge.distance(),
            current: edge.target(),
            parent: ParentEntry {
                parent: s0,
                parent_edge_distance: edge.distance(),
                parent_edge: id.into(),
            },
        });
    }
}

fn border_distance<A>(
    s0: NodeId,
    node: NodeId,
    distance: f32,
    parents: &HashMap<NodeId, A>,
    computed: &ComputedState,
    parent_border_distance: f32,
) -> f32 {
    let min_border_distance = if parents.contains_key(&s0) {
        distance + computed.forward.radius(node)
    } else {
        0.0
    };

    f32::max(min_border_distance, parent_border_distance)
}

fn reference_distance<A>(
    distance: f32,
    border_distance: f32,
    max_parent_reference_distance: f32,
    parents: &HashMap<NodeId, A>,
    visited: &HashMap<NodeId, VisitedState>,
) -> f32 {
    if max_parent_reference_distance == f32::INFINITY && distance > border_distance {
        parents
            .keys()
            .flat_map(|p| visited[p].parents.iter())
            .map(|sp| visited[sp.0].distance)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    } else {
        max_parent_reference_distance
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use crate::{
        highway::dijkstra::{collect_next_level_edges, create_directed_acyclic_graph},
        tests::{create_ref_network_1, create_undirected_network, TestEdge, TestNode},
        DirectedNetworkGraph, EdgeId, NetworkEdge, NodeId,
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
        for id in next_edges {
            let edge = network.edge(id);
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
        for id in next_edges {
            let edge = network.edge(id);
            println!("ID: {:?} - {:?}", id, edge);
        }
    }
}
