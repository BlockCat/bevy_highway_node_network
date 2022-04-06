use crate::{
    BackwardNeighbourhood, DirectedNetworkGraph, EdgeId, ForwardNeighbourhood, NetworkData, NodeId,
};
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

mod core;
mod dijkstra;

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

pub fn phase_1<D: NetworkData>(
    size: usize,
    network: &DirectedNetworkGraph<D>,
) -> HashMap<NodeId, Vec<EdgeId>> {
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
        .flat_map_iter(|(id, _)| {
            dijkstra::calculate_edges(id.into(), &computed, network).into_iter()
        })
        .collect::<HashSet<_>>();
    println!("Finished computing (edges collections)");
    edges.into_iter().into_group_map()
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
