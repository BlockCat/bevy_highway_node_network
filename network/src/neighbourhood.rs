use crate::{DirectedNetworkGraph, NetworkEdge, NetworkNode, NodeId};
use rayon::prelude::*;
use std::{collections::HashMap, ops::Deref};

pub struct ForwardNeighbourhood(Neighbourhood);
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

pub struct PathParent {
    pub distance: f32,
    pub parent: NodeId,
}
pub struct Neighbourhood {
    pub neighbours: Vec<HashMap<NodeId, Option<PathParent>>>,
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
        let neighbours = network
            .nodes
            .par_iter()
            .map(|node| find_forward_neighbourhood(node.id(), size, &network))
            .collect();

        ForwardNeighbourhood(Neighbourhood { neighbours })
    }
}

impl BackwardNeighbourhood {
    pub fn from_network<V: NetworkNode, E: NetworkEdge>(
        size: usize,
        network: &DirectedNetworkGraph<V, E>,
    ) -> Self {
        let neighbours = network
            .nodes
            .par_iter()
            .map(|node| find_backward_neighbourhood(node.id(), size, &network))
            .collect();

        BackwardNeighbourhood(Neighbourhood { neighbours })
    }
}

fn find_forward_neighbourhood<V: NetworkNode, E: NetworkEdge>(
    node: NodeId,
    size: usize,
    network: &DirectedNetworkGraph<V, E>,
) -> HashMap<NodeId, Option<PathParent>> {
    network
        .forward_iterator(node)
        .take(size)
        .map(|(node, distance, edge)| {
            (
                node,
                edge.map(|edge| PathParent {
                    distance,
                    parent: network.edges[*edge].source(),
                }),
            )
        })
        .collect()
}

fn find_backward_neighbourhood<V: NetworkNode, E: NetworkEdge>(
    node: NodeId,
    size: usize,
    network: &DirectedNetworkGraph<V, E>,
) -> HashMap<NodeId, Option<PathParent>> {
    network
        .backward_iterator(node)
        .take(size)
        .map(|(node, distance, edge)| {
            (
                node,
                edge.map(|edge| PathParent {
                    distance,
                    parent: network.edges[*edge].target(),
                }),
            )
        })
        .collect()
}