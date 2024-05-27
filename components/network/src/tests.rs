use crate::{HighwayGraph, IntermediateGraph};

// https://www.baeldung.com/wp-content/uploads/2017/01/initial-graph.png
// pub fn create_ref_network_1() -> HighwayGraph<usize, f32> {
//     let mut graph = IntermediateGraph::default();

//     let nodes = [
//         graph.add_node(0),
//         graph.add_node(1),
//         graph.add_node(2),
//         graph.add_node(3),
//         graph.add_node(4),
//         graph.add_node(5),
//     ];

//     graph.extend_with_edges([
//         (nodes[0], nodes[1], 10.0), // A -> B
//         (nodes[0], nodes[2], 15.0), // A -> C
//         (nodes[1], nodes[3], 12.0), // B -> D
//         (nodes[1], nodes[5], 15.0), // B -> F
//         (nodes[1], nodes[0], 10.0), // A <- B
//         (nodes[2], nodes[4], 10.0), // C -> E
//         (nodes[2], nodes[0], 15.0), // A <- C
//         (nodes[3], nodes[4], 2.0),  // D -> E
//         (nodes[3], nodes[5], 1.0),  // D -> F
//         (nodes[3], nodes[1], 12.0), // B <- D
//         (nodes[4], nodes[2], 10.0), // C <- E
//         (nodes[4], nodes[3], 2.0),  // D <- E
//         (nodes[4], nodes[5], 5.0),  // F <- E
//         (nodes[5], nodes[4], 5.0),  // F -> E
//         (nodes[5], nodes[1], 15.0), // B <- F
//         (nodes[5], nodes[3], 1.0),  // D <- F
//     ]);

//     HighwayGraph::from(graph)
// }

// pub fn create_undirected_network() -> HighwayGraph<usize, f32> {
//     let mut graph = IntermediateGraph::default();
//     let nodes = (0..16).map(|x| graph.add_node(x)).collect::<Vec<_>>();

//     graph.add_edge(nodes[0], nodes[1], 3.0);
//     graph.add_edge(nodes[1], nodes[0], 3.0);
//     graph.add_edge(nodes[1], nodes[2], 2.0);
//     graph.add_edge(nodes[2], nodes[1], 2.0);
//     graph.add_edge(nodes[2], nodes[3], 2.0);
//     graph.add_edge(nodes[3], nodes[2], 2.0);
//     graph.add_edge(nodes[3], nodes[0], 2.0);
//     graph.add_edge(nodes[0], nodes[3], 2.0);
//     graph.add_edge(nodes[7], nodes[6], 2.0);
//     graph.add_edge(nodes[6], nodes[7], 2.0);
//     graph.add_edge(nodes[6], nodes[5], 2.0);
//     graph.add_edge(nodes[5], nodes[6], 2.0);
//     graph.add_edge(nodes[4], nodes[5], 3.0);
//     graph.add_edge(nodes[5], nodes[4], 3.0);
//     graph.add_edge(nodes[4], nodes[7], 2.0);
//     graph.add_edge(nodes[7], nodes[4], 2.0);
//     graph.add_edge(nodes[8], nodes[9], 3.0);
//     graph.add_edge(nodes[9], nodes[8], 3.0);
//     graph.add_edge(nodes[9], nodes[10], 2.0);
//     graph.add_edge(nodes[10], nodes[9], 2.0);
//     graph.add_edge(nodes[10], nodes[11], 2.0);
//     graph.add_edge(nodes[11], nodes[10], 2.0);
//     graph.add_edge(nodes[11], nodes[8], 2.0);
//     graph.add_edge(nodes[8], nodes[11], 2.0);
//     graph.add_edge(nodes[12], nodes[13], 2.0);
//     graph.add_edge(nodes[13], nodes[12], 2.0);
//     graph.add_edge(nodes[13], nodes[14], 2.0);
//     graph.add_edge(nodes[14], nodes[13], 2.0);
//     graph.add_edge(nodes[14], nodes[15], 3.0);
//     graph.add_edge(nodes[15], nodes[14], 3.0);
//     graph.add_edge(nodes[15], nodes[12], 2.0);
//     graph.add_edge(nodes[12], nodes[15], 2.0);
//     graph.add_edge(nodes[12], nodes[16], 6.0);
//     graph.add_edge(nodes[16], nodes[12], 6.0);
//     graph.add_edge(nodes[16], nodes[0], 5.0);
//     graph.add_edge(nodes[0], nodes[16], 5.0);
//     graph.add_edge(nodes[13], nodes[7], 7.0);
//     graph.add_edge(nodes[7], nodes[13], 7.0);
//     graph.add_edge(nodes[2], nodes[4], 6.0);
//     graph.add_edge(nodes[4], nodes[2], 6.0);
//     graph.add_edge(nodes[14], nodes[11], 14.0);
//     graph.add_edge(nodes[11], nodes[14], 14.0);

//     HighwayGraph::from(graph)
// }
