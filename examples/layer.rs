use bevy_dutch_road_highway_node_network::{nwb::NwbGraph, read_file, write_file};
use bevy_shapefile::{RoadId, RoadMap};
use highway::generation::calculate_layer;
use network::{iterators::Distanceable, HighwayGraph, IntermediateGraph};

use petgraph::{algo, Graph};
use rayon::join;

#[derive(Debug, Clone)]
struct RoadWeight(RoadId, f32);

impl Distanceable for RoadWeight {
    fn distance(&self) -> f32 {
        self.1
    }
}

fn main() {
    let (network, road_map) = join(
        || read_file::<NwbGraph, _>("data/directed_graph.graph").unwrap(),
        || read_file::<RoadMap, _>("data/road_map.data").unwrap(),
    );

    println!(
        "Layer: 0 - n: {}, e: {} ",
        network.node_count(),
        network.edge_count()
    );

    let network = network.map(
        |_, n| *n,
        |_, e| {
            let distance = road_map.road_length(*e);
            RoadWeight(*e, distance)
        },
    );

    println!(
        "A- Connected: {} -- {} -- {}",
        algo::connected_components(&Graph::from(network.clone())),
        network.node_count(),
        network.edge_count()
    );

    let network = HighwayGraph::from(network);
    let back = IntermediateGraph::from(network.clone());

    println!(
        "B- Connected: {} -- {} -- {}",
        algo::connected_components(&Graph::from(back.clone())),
        network.node_count(),
        network.edge_count()
    );

    println!("C - ? {} -- {}", back.node_count(), back.edge_count());



    println!("Translated network");
    // panic!();

    let mut layers = vec![calculate_layer(30, network.clone(), 2.0)];

    if let Some(x) = layers.first() {
        write_file(x, "data/0.graph").expect("Could not write");
    }

    for i in 1..7 {
        let prev_layer = layers.last().unwrap();
        println!(
            "Layer: {} - n: {}, e: {} ",
            i,
            prev_layer.node_count(),
            prev_layer.edge_count()
        );
        let next = calculate_layer(30, prev_layer.clone(), 2.0);

        write_file(&next, format!("data/{i}.graph")).expect("Could not write");
        layers.push(next);
    }

    print_data(0, 1, &network, &layers[0]);

    let mut counter = 0;

    for layer in layers.windows(2) {
        counter += 1;
        let l1 = &layer[0];
        let l2 = &layer[1];

        print_data(counter, counter + 1, l1, l2);
    }
}

fn print_data<A, B, C, D>(
    level_a: usize,
    level_b: usize,
    a: &HighwayGraph<A, B>,
    b: &HighwayGraph<C, D>,
) {
    println!("From: {level_a} to {level_b}");
    let nodes_a = a.node_count();
    let nodes_b = b.node_count();
    let percentage = nodes_b as f32 / nodes_a as f32 - 1.0;
    println!(
        "Nodes: {} -> {} {:02}%",
        nodes_a,
        nodes_b,
        percentage * 100.0
    );
}
