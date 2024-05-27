use self::intermediate_network::{IntermediateData, IntermediateNetwork};
use crate::generation::intermediate_network::IntermediateEdge;
use network::{
    BackwardNeighbourhood, DirectedNetworkGraph, ForwardNeighbourhood, NetworkData, ShortcutState,
};
use rayon::prelude::*;
use std::collections::HashSet;

pub mod core;
pub mod dag;
pub mod dijkstra;
pub mod intermediate_network;

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
    let (duration, intermediate) = stopwatch!(phase_1(size, network));

    println!("Finished phase 1 {}ms", duration.as_millis());

    let (duration, intermediate) = stopwatch!(phase_2(intermediate, contraction_factor));
    println!("Finished phase 2 {}ms", duration.as_millis());

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
        "Start computing (nodes: {}, edges collections: {})",
        network.nodes().len(),
        network.edges().len()
    );

    let (duration, edges) = stopwatch!(network
        .nodes()
        .par_iter()
        .enumerate()
        .flat_map_iter(|(id, _)| dijkstra::calculate_edges(id.into(), &computed, network))
        .collect::<HashSet<_>>());

    println!(
        "Finished computing (edges collections) {}ms",
        duration.as_millis()
    );

    let edges = edges
        .into_iter()
        .map(|(source, edge_id)| {
            let edge = network.edge(edge_id);
            IntermediateEdge::new(
                source,
                edge.target(),
                edge.distance(),
                ShortcutState::Single(edge.edge_id),
                network.data.edge_road_id(edge_id),
                network::builder::EdgeDirection::Forward,
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
