use std::collections::HashSet;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod dijkstra;

use crate::{
    BackwardNeighbourhood, DirectedNetworkGraph, ForwardNeighbourhood, NetworkEdge, NetworkNode,
};

pub fn phase_1<V: NetworkNode, E: NetworkEdge>(size: usize, network: &DirectedNetworkGraph<V, E>) -> HashSet<crate::EdgeId> {
    let computed = ComputedState::new(size, network);


    network
        .nodes
        .par_iter()
        .flat_map_iter(|node| dijkstra::calculate_edges(node.id(), &computed, network).into_iter())
        .collect::<HashSet<_>>()
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
