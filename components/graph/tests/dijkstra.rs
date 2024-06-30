use graph::create_network;

#[test]
fn forward_dijkstra_test() {
    // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
    let network = create_network!(
        0..5,
        0 => 1; 10.0, // A => B
        0 => 2; 15.0, // A => C
        1 => 3; 12.0, // B => D
        1 => 5; 15.0, // B => F
        2 => 4; 10.0, // C => E
        3 => 4; 2.0, // D => E
        3 => 5; 1.0, // D => F
        5 => 4; 5.0 // F => E
    );

    let forward = (0..=5u32)
        .map(|i| network.forward_iterator(i.into()).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    assert_eq!(
        forward[0],
        vec![
            (0u32.into(), 0.0f32),
            (1u32.into(), 10.0f32),
            (2u32.into(), 15.0f32),
            (3u32.into(), 22.0f32),
            (5u32.into(), 23.0f32),
            (4u32.into(), 24.0f32),
        ]
    );
    assert_eq!(
        forward[1],
        vec![
            (1u32.into(), 0.0f32),
            (3u32.into(), 12.0f32),
            (5u32.into(), 13.0f32),
            (4u32.into(), 14.0f32),
        ]
    );
    assert_eq!(
        forward[2],
        vec![(2u32.into(), 0.0f32), (4u32.into(), 10.0f32),]
    );

    assert_eq!(
        forward[3],
        vec![
            (3u32.into(), 0.0f32),
            (5u32.into(), 1.0f32),
            (4u32.into(), 2.0f32),
        ]
    );
    assert_eq!(forward[4], vec![(4u32.into(), 0.0f32)]);
    assert_eq!(
        forward[5],
        vec![(5u32.into(), 0.0f32), (4u32.into(), 5.0f32)]
    );
}

#[test]
fn backward_dijkstra_test() {
    // https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
    let network = create_network!(
        0..5,
        0 => 1; 10.0, // A => B
        0 => 2; 15.0, // A => C
        1 => 3; 12.0, // B => D
        1 => 5; 15.0, // B => F
        2 => 4; 10.0, // C => E
        3 => 4; 2.0, // D => E
        3 => 5; 1.0, // D => F
        5 => 4; 5.0 // F => E
    );

    let backward = (0..=5u32)
        .map(|i| network.backward_iterator(i.into()).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    assert_eq!(backward[0], vec![(0u32.into(), 0.0f32),]);
    assert_eq!(
        backward[1],
        vec![(1u32.into(), 0.0f32), (0u32.into(), 10.0f32),]
    );

    assert_eq!(
        backward[2],
        vec![(2u32.into(), 0.0f32), (0u32.into(), 15.0f32),]
    );

    assert_eq!(
        backward[3],
        vec![
            (3u32.into(), 0.0f32),
            (1u32.into(), 12.0f32),
            (0u32.into(), 22.0f32),
        ]
    );
    assert_eq!(
        backward[4],
        vec![
            (4u32.into(), 0.0f32),
            (3u32.into(), 2.0f32),
            (5u32.into(), 5.0f32),
            (2u32.into(), 10.0f32),
            (1u32.into(), 14.0f32),
            (0u32.into(), 24.0f32),
        ]
    );
    assert_eq!(
        backward[5],
        vec![
            (5u32.into(), 0.0f32),
            (3u32.into(), 1.0f32),
            (1u32.into(), 13.0f32),
            (0u32.into(), 23.0f32),
        ]
    );
}
