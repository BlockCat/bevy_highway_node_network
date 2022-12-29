use network::{
    iterators::Distanceable, BackwardNeighbourhood, ForwardNeighbourhood, HighwayGraph, Shorted,
};
use rayon::prelude::*;
use std::collections::HashSet;
use petgraph::visit::*;

pub mod core;
pub mod dag;
pub mod dijkstra;
// pub mod intermediate_network;

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

/// Calculate the next layer from a network.
/// Phase 1: Generate highway network
/// Phase 2: Create a core graph and bypass nodes
pub fn calculate_layer<N, E>(
    size: usize,
    network: HighwayGraph<N, E>,
    contraction_factor: f32,
) -> HighwayGraph<N, Shorted>
where
    N: Send + Sync + Clone,
    E: Send + Sync + Distanceable,
{
    let phase_1_graph = phase_1(size, network);

    phase_2(phase_1_graph, contraction_factor)
}

/// Phase 1: ... ?
pub(crate) fn phase_1<N: Send + Sync, E: Send + Sync + Distanceable>(
    size: usize,
    mut network: HighwayGraph<N, E>,
) -> HighwayGraph<N, E> {
    println!("Start computing (forward backward)");

    let (duration, computed) = stopwatch!(ComputedState::new(size, &network));

    println!(
        "Finished computing (forward + backward) {}ms",
        duration.as_millis()
    );
    println!(
        "Start computing (edges collections: {})",
        network.edge_count()
    );

    let edges = network
        .node_identifiers()
        .par_bridge()
        .flat_map_iter(|id| dijkstra::calculate_edges(id, &computed, &network))
        .collect::<HashSet<_>>();

    println!("Got retained edges");

    network.retain_edges(|_, e| edges.contains(&e));

    println!("Finished computing (edges collections)");

    network
}

/**
 * Calculate the core network
 */
fn phase_2<N, E>(
    intermediate: HighwayGraph<N, E>,
    contraction_factor: f32,
) -> HighwayGraph<N, Shorted>
where
    N: Clone,
    E: Distanceable,
{
    core::core_network_with_patch(intermediate, contraction_factor)
}

pub struct ComputedState {
    pub forward: ForwardNeighbourhood,
    pub backward: BackwardNeighbourhood,
}

impl ComputedState {
    pub fn new<N: Send + Sync, E: Send + Sync + Distanceable>(
        size: usize,
        network: &HighwayGraph<N, E>,
    ) -> Self {
        let (forward, backward) = rayon::join(
            || ForwardNeighbourhood::from_network(size, network),
            || BackwardNeighbourhood::from_network(size, network),
        );

        ComputedState { forward, backward }
    }
}
