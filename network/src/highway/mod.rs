use std::collections::HashSet;

use self::intermediate_network::{IntermediateData, IntermediateNetwork};
use crate::{
    highway::intermediate_network::IntermediateEdge, BackwardNeighbourhood, DirectedNetworkGraph,
    ForwardNeighbourhood, NetworkData, ShortcutState,
};
use rayon::prelude::*;

pub mod core;
pub mod dijkstra;
pub mod intermediate_network;
pub mod dag;

macro_rules! stopwatch {
    ($x:expr) => {{
        let start = std::time::Instant::now();
        let value = $x;
        let end = std::time::Instant::now();

        (end - start, value)
    }};
    ($x:block) => {{
        let start = std::time::Instant::now();
        let value = $x;
        let end = std::time::Instant::now();

        (end - start, value)
    }};

    (print $x:block) => {{
        let (duration, value) = stopwatch!($x);

        println!("Duration: {}µs", duration.as_micros());

        value
    }};

    (print $x:expr) => {{
        let (duration, value) = stopwatch!($x);

        println!("Duration: {}µs", duration.as_micros());

        value
    }};
}

pub fn calculate_layer<D: NetworkData>(
    size: usize,
    network: &DirectedNetworkGraph<D>,
    contraction_factor: f32,
) -> DirectedNetworkGraph<IntermediateData> {
    let intermediate = phase_1(size, network);

    // let intermediate = phase_2(intermediate, contraction_factor);

    DirectedNetworkGraph::from(intermediate)
}
pub(crate) fn phase_1<D: NetworkData>(
    size: usize,
    network: &DirectedNetworkGraph<D>,
) -> IntermediateNetwork {
    println!("Start computing (forward backward)");

    let (duration, computed) = stopwatch!(ComputedState::new(size, network));

    println!(
        "Finished computing (forward backward) {}ms",
        duration.as_millis()
    );
    println!(
        "Start computing (edges collections: {})",
        network.edges().len()
    );

    let edges = network
        .nodes()
        .par_iter()
        .enumerate()
        .flat_map_iter(|(id, _)| dijkstra::calculate_edges(id.into(), &computed, network))
        .collect::<HashSet<_>>();

    let edges = edges
        .into_iter()
        .map(|(source, edge_id)| {
            let edge = network.edge(edge_id);
            IntermediateEdge::new(
                source,
                edge.target(),
                edge.distance(),
                ShortcutState::Single(edge.edge_id),
                crate::builder::EdgeDirection::Forward,
            )
        })
        .collect();
    println!("Finished computing (edges collections)");

    edges
}

/**
 * Calculate the core network
 */
fn phase_2(intermediate: IntermediateNetwork, contraction_factor: f32) -> IntermediateNetwork {
    core::core_network_with_patch(intermediate, contraction_factor)
}

pub struct ComputedState {
    pub forward: ForwardNeighbourhood,
    pub backward: BackwardNeighbourhood,
}

impl ComputedState {
    pub fn new<D: NetworkData>(size: usize, network: &DirectedNetworkGraph<D>) -> Self {
        let (forward, backward) = rayon::join(
            || ForwardNeighbourhood::from_network(size, network),
            || BackwardNeighbourhood::from_network(size, network),
        );

        ComputedState { forward, backward }
    }
}
