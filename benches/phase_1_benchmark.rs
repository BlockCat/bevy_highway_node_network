use bevy_dutch_road_highway_node_network::{nwb::NWBNetworkData, read_file};
use criterion::{criterion_group, criterion_main, Criterion};
use network::DirectedNetworkGraph;

fn bench(b: &mut Criterion) {
    let network: DirectedNetworkGraph<NWBNetworkData> =
        read_file("data/directed_graph.graph").unwrap();

    let mut group = b.benchmark_group("sample: 10");

    group.sample_size(10);
    group.bench_function("network::phase_1", |b| {
        b.iter(|| network::phase_1(30, &network));
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
