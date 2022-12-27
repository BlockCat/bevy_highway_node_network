use bevy_dutch_road_highway_node_network::{nwb::NWBNetworkData, read_file, write_file};
use highway::generation::calculate_layer;
use network::{DirectedNetworkGraph, NetworkData};

fn main() {
    let network: DirectedNetworkGraph<NWBNetworkData> =
        read_file("data/directed_graph.graph").unwrap();

    println!(
        "Layer: 0 - n: {}, e: {} ",
        network.nodes().len(),
        network.edges().len()
    );

    let mut layers = vec![calculate_layer(30, &network, 2.0)];

    if let Some(x) = layers.first() {
        write_file(x, "data/0.graph".to_string()).expect("Could not write");
    }

    for i in 1..7 {
        let prev_layer = layers.last().unwrap();
        println!(
            "Layer: {} - n: {}, e: {} ",
            i,
            prev_layer.nodes().len(),
            prev_layer.edges().len()
        );
        let next = calculate_layer(30, prev_layer, 3.0);

        write_file(&next, format!("data/{i}.graph")).expect("Could not write");
        layers.push(next);
    }

    data(0, 1, &network, &layers[0]);

    let mut counter = 0;

    for layer in layers.windows(2) {
        counter += 1;
        let l1 = &layer[0];
        let l2 = &layer[1];

        data(counter, counter + 1, l1, l2);
    }
}

fn data<A: NetworkData, B: NetworkData>(
    level_a: usize,
    level_b: usize,
    a: &DirectedNetworkGraph<A>,
    b: &DirectedNetworkGraph<B>,
) {
    println!("From: {level_a} to {level_b}");
    let nodes_a = a.nodes().len();
    let nodes_b = b.nodes().len();
    let percentage = nodes_b as f32 / nodes_a as f32 - 1.0;
    println!(
        "Nodes: {} -> {} {:02}%",
        nodes_a,
        nodes_b,
        percentage * 100.0
    );
}
