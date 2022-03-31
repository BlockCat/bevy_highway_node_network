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
        create_network,
        tests::{TestEdge, TestNode},
        DirectedNetworkGraph, EdgeId, ForwardNeighbourhood, NodeId, PathParent,
    };

    #[test]
    fn forward_neighbourhood_test() {
        // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
        let network = create_network!(
            0 => 1; 10.0,
            0 => 2; 15.0,
            1 => 3; 5.0,
            1 => 5; 15.0,
            2 => 4; 10.0,
            3 => 4; 2.0,
            3 => 5; 1.0,
            5 => 4; 5.0
        );

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
