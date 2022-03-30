use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    BackwardNeighbourhood, DirectedNetworkGraph, ForwardNeighbourhood, NetworkEdge, NetworkNode,
};

pub fn phase_1<V: NetworkNode, E: NetworkEdge>(size: usize, network: &DirectedNetworkGraph<V, E>) {
    let computed = ComputedState::new(size, network);

    network
        .nodes
        .par_iter()
        .map(|node| dijkstra::forward(node.id(), &computed, network))
        .collect::<Vec<_>>();
}

pub struct ComputedState {
    pub forward: ForwardNeighbourhood,
    pub backward: BackwardNeighbourhood,
}

impl ComputedState {
    pub fn new<V: NetworkNode, E: NetworkEdge>(
        size: usize,
        network: &DirectedNetworkGraph<V, E>,
    ) -> Self {
        let (forward, backward) = rayon::join(
            || ForwardNeighbourhood::from_network(size, network),
            || BackwardNeighbourhood::from_network(size, network),
        );

        ComputedState { forward, backward }
    }
}

mod dijkstra {
    use std::collections::{BinaryHeap, HashMap, HashSet};

    use crate::{DirectedNetworkGraph, NetworkEdge, NetworkNode, NodeId};

    use super::ComputedState;

    #[derive(Debug, PartialEq)]
    struct DijkstraNodeState {
        distance: f32,
        current: NodeId,
        parent: NodeId,
    }

    struct VisitedState {
        border_distance: f32,
        reference_distance: f32,
        distance: f32,
        parents: HashSet<NodeId>,
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

    pub fn forward<V: NetworkNode, E: NetworkEdge>(
        s0: NodeId,
        computed: &ComputedState,
        network: &DirectedNetworkGraph<V, E>,
    ) {
        let mut heap: BinaryHeap<DijkstraNodeState> = BinaryHeap::new();

        let mut visited: HashMap<NodeId, VisitedState> = HashMap::new();

        initialize_heap(s0, network, &mut heap, &mut visited);

        while let Some(mut state) = heap.pop() {
            if visited.contains_key(&state.current) {
                debug_assert!(state.distance > visited[&state.current].distance);
                continue;
            }

            let mut parents = HashSet::from([state.parent]);

            let mut parent_border_distance = visited[&state.parent].border_distance;
            let mut parent_reference_distance = visited[&state.parent].reference_distance;

            while let Some(peek) = heap
                .peek()
                .filter(|next| next.current == state.current && next.distance == state.distance)
            {
                let parent_visited = &visited[&peek.parent];
                parent_border_distance =
                    f32::max(parent_border_distance, parent_visited.border_distance);

                parent_reference_distance =
                    f32::max(parent_reference_distance, parent_visited.reference_distance);

                parents.insert(peek.parent);
                state = heap.pop().unwrap();
            }

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

            let should_abort =
                (reference_distance + computed.backward.radius[*state.current]) < state.distance;

            visited.insert(
                state.current,
                VisitedState {
                    distance: state.distance,
                    border_distance: border_distance,
                    reference_distance,
                    parents,
                },
            );

            if !should_abort {
                for child_edge in network.out_edges[*state.current]
                    .iter()
                    .map(|eid| &network.edges[**eid])
                {
                    let child = child_edge.target();
                    let next_distance = state.distance + child_edge.distance();

                    heap.push(DijkstraNodeState {
                        current: child,
                        parent: state.current,
                        distance: next_distance,
                    });
                }
            } else {
                println!("ABORTED: {:?}", state);
            }
        }

        println!("DAG B:");

        for (node, visited) in visited {
            println!(
                "Node: {}, dist: {}, parents: {:?}",
                node.0, visited.distance, visited.parents
            );
        }
    }

    fn initialize_heap<V: NetworkNode, E: NetworkEdge>(
        s0: NodeId,
        network: &DirectedNetworkGraph<V, E>,
        heap: &mut BinaryHeap<DijkstraNodeState>,
        visited: &mut HashMap<NodeId, VisitedState>,
    ) {
        visited.insert(
            s0,
            VisitedState {
                border_distance: 0.0,
                reference_distance: f32::INFINITY,
                distance: 0.0,
                parents: HashSet::from([s0]),
            },
        );
        for id in &network.out_edges[*s0] {
            let edge = &network.edges[**id];

            heap.push(DijkstraNodeState {
                distance: edge.distance(),
                current: edge.target(),
                parent: s0,
            });
        }
    }

    fn border_distance(
        s0: NodeId,
        node: NodeId,
        distance: f32,
        parents: &HashSet<NodeId>,
        computed: &ComputedState,
        parent_border_distance: f32,
    ) -> f32 {
        let min_border_distance = if parents.contains(&s0) {
            distance + computed.forward.radius[*node]
        } else {
            0.0
        };

        f32::max(min_border_distance, parent_border_distance)
    }

    fn reference_distance(
        distance: f32,
        border_distance: f32,
        max_parent_reference_distance: f32,
        parents: &HashSet<NodeId>,
        visited: &HashMap<NodeId, VisitedState>,
    ) -> f32 {
        if max_parent_reference_distance == f32::INFINITY && distance > border_distance {
            parents
                .iter()
                .flat_map(|p| visited[p].parents.iter())
                .map(|sp| visited[sp].distance)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        } else {
            max_parent_reference_distance
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{DirectedNetworkGraph, EdgeId, NetworkEdge, NetworkNode, NodeId};

    use super::ComputedState;

    #[derive(Debug, Clone, Copy)]
    struct TestNode(usize);

    struct TestEdge(usize, usize, f32);

    impl NetworkNode for TestNode {
        fn id(&self) -> crate::NodeId {
            self.0.into()
        }
    }

    impl NetworkEdge for TestEdge {
        fn source(&self) -> crate::NodeId {
            self.0.into()
        }

        fn target(&self) -> crate::NodeId {
            self.1.into()
        }

        fn distance(&self) -> f32 {
            self.2
        }
    }

    // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
    fn create_network() -> DirectedNetworkGraph<TestNode, TestEdge> {
        DirectedNetworkGraph {
            nodes: vec![
                TestNode(0), // A
                TestNode(1), // B
                TestNode(2), // C
                TestNode(3), // D
                TestNode(4), // E
                TestNode(5), // F
            ],
            edges: vec![
                TestEdge(0, 1, 10.0), // A -> B | 0
                TestEdge(0, 2, 15.0), // A -> C | 1
                TestEdge(1, 3, 12.0), // B -> D | 2
                TestEdge(1, 5, 15.0), // B -> F | 3
                TestEdge(2, 4, 10.0), // C -> E | 4
                TestEdge(3, 4, 2.0),  // D -> E | 5
                TestEdge(3, 5, 1.0),  // D -> F | 6
                TestEdge(5, 4, 5.0),  // F -> E | 7
            ],
            out_edges: vec![
                vec![EdgeId(0), EdgeId(1)],
                vec![EdgeId(2), EdgeId(3)],
                vec![EdgeId(4)],
                vec![EdgeId(5), EdgeId(6)],
                vec![],
                vec![EdgeId(7)],
            ],
            in_edges: vec![
                vec![],                                // A
                vec![EdgeId(0)],                       // B
                vec![EdgeId(1)],                       // C
                vec![EdgeId(2)],                       // D
                vec![EdgeId(4), EdgeId(5), EdgeId(7)], // E
                vec![EdgeId(3), EdgeId(6)],            // F
            ],
        }
    }

    #[test]
    fn forward_test() {
        let network = create_network();
        let computed = ComputedState::new(3, &network);

        println!("\nRadius: {:?}", computed.forward.radius);

        println!("\nForward:");
        for ele in computed.forward.neighbours.iter().enumerate() {
            println!("{}, {:?}", ele.0, ele.1.keys());
        }

        println!("\nBackward:");
        for ele in computed.backward.neighbours.iter().enumerate() {
            println!("{}, {:?}", ele.0, ele.1.keys());
        }

        super::dijkstra::forward(NodeId(0), &computed, &network);
    }
}
