use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod dijkstra;

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
