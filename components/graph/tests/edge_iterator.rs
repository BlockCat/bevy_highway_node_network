use graph::create_network;

#[test]
fn out_edge_test() {
    let network = create_network!(
        0..3,
        0 => 1; 10.0, // A => B
        0 => 2; 15.0, // A => C
        1 => 3; 12.0, // B => D
        3 => 0; 15.0, // B => F
        2 => 0; 10.0 // C => E

    );

    let out_edges = network.out_edges(0u32.into()).collect::<Vec<_>>();
    assert_eq!(out_edges.len(), 2);

    assert_eq!(out_edges[0].1.target(), 1u32.into());
    assert_eq!(out_edges[1].1.target(), 2u32.into());
}

#[test]
fn in_edge_test() {
    let network = create_network!(
        0..3,
        0 => 1; 10.0, // A => B
        0 => 2; 15.0, // A => C
        1 => 3; 12.0, // B => D
        3 => 0; 15.0, // B => F
        2 => 0; 10.0 // C => E
    );

    let out_edges = network.in_edges(0u32.into()).collect::<Vec<_>>();
    assert_eq!(out_edges.len(), 2);

    assert_eq!(out_edges[0].1.target(), 2u32.into());
    assert_eq!(out_edges[1].1.target(), 3u32.into());
}
