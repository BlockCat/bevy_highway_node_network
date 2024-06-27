use bevy_dutch_road_highway_node_network::{nwb::NWBNetworkData, read_file};
use criterion::{criterion_group, criterion_main, Criterion};
use highway::generation::calculate_layer;
use graph::DirectedNetworkGraph;

fn bench(b: &mut Criterion) {
    let network: DirectedNetworkGraph<NWBNetworkData> =
        read_file("data/directed_graph.graph").unwrap();

    let mut group = b.benchmark_group("sample: 10");

    group.sample_size(10);
    group.bench_function("network::phase_1", |b| {
        b.iter(|| calculate_layer(30, &network, 2.0));
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
