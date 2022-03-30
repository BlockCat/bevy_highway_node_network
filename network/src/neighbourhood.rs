use crate::{DirectedNetworkGraph, NetworkEdge, NetworkNode, NodeId};
use rayon::prelude::*;
use std::{collections::HashMap, ops::Deref};

#[derive(Debug)]
pub struct ForwardNeighbourhood(Neighbourhood);

#[derive(Debug)]
pub struct BackwardNeighbourhood(Neighbourhood);

impl Deref for ForwardNeighbourhood {
    type Target = Neighbourhood;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for BackwardNeighbourhood {
    type Target = Neighbourhood;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct PathParent {
    pub distance: f32,
    pub parent: NodeId,
}

impl PathParent {
    pub fn new(distance: f32, parent: NodeId) -> Self {
        Self { distance, parent }
    }
}

#[derive(Debug)]
pub struct Neighbourhood {
    pub neighbours: Vec<HashMap<NodeId, Option<PathParent>>>,
    pub radius: Vec<f32>,
}

impl Neighbourhood {
    pub fn contains(&self, node: NodeId, neigbhour: NodeId) -> bool {
        self.neighbours[*node].contains_key(&neigbhour)
    }

    pub fn get(&self, node: NodeId, neigbhour: NodeId) -> Option<&PathParent> {
        self.neighbours[*node]
            .get(&neigbhour)
            .and_then(Option::as_ref)
    }
}

impl ForwardNeighbourhood {
    pub fn from_network<V: NetworkNode, E: NetworkEdge>(
        size: usize,
        network: &DirectedNetworkGraph<V, E>,
    ) -> Self {
        let mut radius = Vec::with_capacity(network.nodes.len());
        let mut neighbours = Vec::with_capacity(network.nodes.len());

        network
            .nodes
            .par_iter()
            .map(|node| find_forward_neighbourhood(node.id(), size, &network))
            .unzip_into_vecs(&mut radius, &mut neighbours);

        ForwardNeighbourhood(Neighbourhood { neighbours, radius })
    }
}

impl BackwardNeighbourhood {
    pub fn from_network<V: NetworkNode, E: NetworkEdge>(
        size: usize,
        network: &DirectedNetworkGraph<V, E>,
    ) -> Self {
        let mut radius = Vec::with_capacity(network.nodes.len());
        let mut neighbours = Vec::with_capacity(network.nodes.len());

        network
            .nodes
            .par_iter()
            .map(|node| find_backward_neighbourhood(node.id(), size, &network))
            .unzip_into_vecs(&mut radius, &mut neighbours);

        BackwardNeighbourhood(Neighbourhood { neighbours, radius })
    }
}

fn find_forward_neighbourhood<V: NetworkNode, E: NetworkEdge>(
    node: NodeId,
    size: usize,
    network: &DirectedNetworkGraph<V, E>,
) -> (f32, HashMap<NodeId, Option<PathParent>>) {
    let mut iterator = network.forward_iterator(node);

    let map = iterator.by_ref().take(size).collect::<Vec<_>>();
    let distance = iterator.distance;

    let map = map
        .into_iter()
        .chain(iterator.take_while(|x| x.1 <= distance))
        .map(|(node, distance, edge)| {
            (
                node,
                edge.map(|edge| PathParent {
                    distance,
                    parent: network.edges[*edge].source(),
                }),
            )
        })
        .collect::<HashMap<_, _>>();

    (distance, map)
}

fn find_backward_neighbourhood<V: NetworkNode, E: NetworkEdge>(
    node: NodeId,
    size: usize,
    network: &DirectedNetworkGraph<V, E>,
) -> (f32, HashMap<NodeId, Option<PathParent>>) {
    let mut iterator = network.backward_iterator(node);

    let map = iterator.by_ref().take(size).collect::<Vec<_>>();
    let distance = iterator.distance;

    let map = map
        .into_iter()
        .chain(iterator.take_while(|x| x.1 <= distance))
        .map(|(node, distance, edge)| {
            (
                node,
                edge.map(|edge| PathParent {
                    distance,
                    parent: network.edges[*edge].target(),
                }),
            )
        })
        .collect();

    (distance, map)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        DirectedNetworkGraph, EdgeId, ForwardNeighbourhood, NetworkEdge, NetworkNode, NodeId,
        PathParent,
    };

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
                TestEdge(1, 3, 5.0),  // B -> D | 2
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
    fn forward_neighbourhood_test() {
        let network = create_network();

        let forward = ForwardNeighbourhood::from_network(3, &network);

        let radius = &forward.radius;

        assert_eq!(&vec![15.0, 6.0, 10.0, 2.0, 0.0, 5.0], radius);

        let network = &forward.neighbours;

        assert_eq!(
            HashMap::from([
                (NodeId(0), None),
                (NodeId(1), Some(PathParent::new(10.0, NodeId(0)))),
                (NodeId(2), Some(PathParent::new(15.0, NodeId(0)))),
                (NodeId(3), Some(PathParent::new(15.0, NodeId(1)))),
            ]),
            network[0]
        );

        assert_eq!(
            HashMap::from([
                (NodeId(1), None),
                (NodeId(3), Some(PathParent::new(5.0, NodeId(1)))),
                (NodeId(5), Some(PathParent::new(6.0, NodeId(3)))),
            ]),
            network[1]
        );

        assert_eq!(
            HashMap::from([
                (NodeId(2), None),
                (NodeId(4), Some(PathParent::new(10.0, NodeId(2)))),
            ]),
            network[2]
        );

        assert_eq!(
            HashMap::from([
                (NodeId(3), None),
                (NodeId(4), Some(PathParent::new(2.0, NodeId(3)))),
                (NodeId(5), Some(PathParent::new(1.0, NodeId(3)))),
            ]),
            network[3]
        );

        assert_eq!(HashMap::from([(NodeId(4), None),]), network[4]);

        assert_eq!(
            HashMap::from([
                (NodeId(5), None),
                (NodeId(4), Some(PathParent::new(5.0, NodeId(5)))),
            ]),
            network[5]
        );
    }
}
