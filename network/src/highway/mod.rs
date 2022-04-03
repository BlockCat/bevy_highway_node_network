use crate::{
    BackwardNeighbourhood, DirectedNetworkGraph, ForwardNeighbourhood, NetworkEdge, NetworkNode,
};
use std::collections::HashSet;

mod dijkstra;

pub fn phase_1<V: NetworkNode, E: NetworkEdge>(
    size: usize,
    network: &DirectedNetworkGraph<V, E>,
) -> HashSet<crate::EdgeId> {
    println!("Start computing (forward backward)");
    let computed = ComputedState::new(size, network);

    println!("Finished computing (forward backward)");
    println!(
        "Start computing (edges collections: {})",
        network.nodes.len()
    );

    let edges = network
        .nodes
        .iter()
        .enumerate()
        .inspect(|x| {
            if x.0 % 1000 == 0 {
                println!("{:?}", x.0);
            }
        })
        .map(|x| x.1)
        .flat_map(|node| dijkstra::calculate_edges(node.id(), &computed, network).into_iter())
        .collect::<HashSet<_>>();
    println!("Finished computing (edges collections)");
    edges
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
