use itertools::Itertools;
use rayon::prelude::*;

use crate::{
    DirectedNetworkGraph, EdgeDirection, EdgeId, NetworkEdge, NetworkNode, NodeId, ShortcutState,
};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Debug)]
pub struct Edge {
    target: NodeId,
    source: NodeId,
    weight: f32,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct DirectedMapGraph {
    out_edges: HashMap<NodeId, Vec<Edge>>,
    in_edges: HashMap<NodeId, Vec<Edge>>,
    edge_count: usize,
}

impl DirectedMapGraph {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn nodes(&self) -> Vec<NodeId> {
        self.out_edges.keys().cloned().collect()
    }

    pub fn out_edges(&self, node: NodeId) -> &Vec<Edge> {
        &self.out_edges[&node]
    }

    pub fn add_node(&mut self) -> NodeId {
        let index = NodeId(self.out_edges.len() as u32);
        self.out_edges.insert(index, Vec::new());
        self.in_edges.insert(index, Vec::new());

        index
    }

    pub fn add_edge(&mut self, source: NodeId, target: NodeId, weight: f32) {
        let id = EdgeId(self.edge_count as u32);

        self.out_edges.get_mut(&source).unwrap().push(Edge {
            target,
            source,
            weight,
        });
        self.in_edges.get_mut(&target).unwrap().push(Edge {
            target,
            source,
            weight,
        });
        self.edge_count += 1;
    }
}

impl From<DirectedNetworkGraph> for DirectedMapGraph {
    fn from(graph: DirectedNetworkGraph) -> Self {
        let mut new_graph = Self::new();
        for _ in 0..graph.nodes().len() {
            new_graph.add_node();
        }
        for (id, _) in graph.nodes().into_iter().enumerate() {
            for (_, edge) in graph.out_edges(NodeId(id as u32)) {
                new_graph.add_edge(NodeId(id as u32), edge.target(), edge.weight());
            }
        }

        new_graph
    }
}

impl From<DirectedMapGraph> for DirectedNetworkGraph {
    fn from(mut value: DirectedMapGraph) -> Self {
        let mut nodes = Vec::with_capacity(value.out_edges.len());
        let mut edges = Vec::with_capacity(value.edge_count);

        for node_id in (0..value.out_edges.len()).map(|i| NodeId(i as u32)) {
            let out_edges = value.out_edges.remove(&node_id).unwrap();
            let in_edges = value.in_edges.remove(&node_id).unwrap();

            let start_edge_index = edges.len() as u32;

            let (out_edges, overlaping_edges, in_edges) = collect_edges(in_edges, out_edges);

            edges.extend(extend_edges(
                start_edge_index,
                out_edges,
                EdgeDirection::Forward,
                |e| e.target,
            ));
            edges.extend(extend_edges(
                edges.len() as u32,
                overlaping_edges,
                EdgeDirection::Both,
                |e| e.target,
            ));
            edges.extend(extend_edges(
                edges.len() as u32,
                in_edges,
                EdgeDirection::Backward,
                |e| e.source,
            ));

            let last_edge_index = edges.len() as u32;

            nodes.push(NetworkNode::new(start_edge_index, last_edge_index));
        }

        DirectedNetworkGraph::new(nodes, edges)
    }
}

fn collect_edges(
    mut in_edges: Vec<Edge>,
    mut out_edges: Vec<Edge>,
) -> (Vec<Edge>, Vec<Edge>, Vec<Edge>) {
    // Find overlapping edges
    let overlaping_edges = out_edges
        .iter()
        .filter(|e| {
            in_edges
                .iter()
                .any(|ie| ie.source == e.target && ie.target == e.source)
        })
        .cloned()
        .collect::<Vec<_>>();

    // Remove overlapping edges from in_edges
    in_edges.retain(|ie| {
        !overlaping_edges
            .iter()
            .any(|e| e.source == ie.target && e.target == ie.source)
    });

    // Remove overlapping edges from out_edges
    out_edges.retain(|e| {
        !overlaping_edges
            .iter()
            .any(|ie| ie.source == e.source && ie.target == e.target)
    });

    (out_edges, overlaping_edges, in_edges)
}

fn extend_edges<F>(
    offset: u32,
    edges: Vec<Edge>,
    direction: EdgeDirection,
    target_node: F,
) -> impl Iterator<Item = NetworkEdge>
where
    F: Fn(&Edge) -> NodeId,
{
    edges.into_iter().enumerate().map(move |(i, e)| {
        NetworkEdge::new(offset + i as u32, target_node(&e), e.weight, direction)
    })
}

pub trait EdgeBuilder {
    fn source(&self) -> NodeId;
    fn target(&self) -> NodeId;
    fn weight(&self) -> f32;
    fn road_id(&self) -> ShortcutState<usize>;
}

impl<E> From<Vec<E>> for DirectedMapGraph
where
    E: EdgeBuilder,
{
    fn from(edges: Vec<E>) -> Self {
        let mut graph = DirectedMapGraph::new();
        let nodes = edges
            .iter()
            .map(|edge| [edge.source(), edge.target()])
            .flatten()
            .sorted_by_key(|n| n.0)
            .collect::<Vec<_>>();

        let convert = nodes
            .into_iter()
            .map(|n| (n, graph.add_node()))
            .collect::<HashMap<_, _>>();

        for edge in edges {
            graph.add_edge(
                convert[&edge.source()],
                convert[&edge.target()],
                edge.weight(),
            );
        }

        graph
    }
}

impl<E> FromIterator<E> for DirectedMapGraph
where
    E: EdgeBuilder,
{
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        let edges = iter.into_iter().collect::<Vec<_>>();
        edges.into()
    }
}

impl<E> FromParallelIterator<E> for DirectedMapGraph
where
    E: EdgeBuilder + Send + Sync,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = E>,
    {
        let edges = par_iter.into_par_iter().collect::<Vec<_>>();
        edges.into()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn build_graph() -> DirectedMapGraph {
        let mut graph = DirectedMapGraph::new();

        for _ in 0..5 {
            graph.add_node();
        }

        graph.add_edge(NodeId(0), NodeId(1), 1.0);
        graph.add_edge(NodeId(0), NodeId(2), 2.0);
        graph.add_edge(NodeId(0), NodeId(3), 3.0);
        graph.add_edge(NodeId(1), NodeId(4), 3.0);
        graph.add_edge(NodeId(2), NodeId(4), 4.0);
        graph.add_edge(NodeId(4), NodeId(3), 4.0);
        graph.add_edge(NodeId(4), NodeId(1), 3.0);

        graph
    }

    #[test]
    fn test_directed_map_graph() {
        let graph = build_graph();

        assert_eq!(graph.out_edges[&NodeId(0)].len(), 3);
        assert_eq!(graph.out_edges[&NodeId(1)].len(), 1);
        assert_eq!(graph.out_edges[&NodeId(2)].len(), 1);
        assert_eq!(graph.out_edges[&NodeId(3)].len(), 0);
        assert_eq!(graph.out_edges[&NodeId(4)].len(), 2);

        assert_eq!(graph.in_edges[&NodeId(0)].len(), 0);
        assert_eq!(graph.in_edges[&NodeId(1)].len(), 2);
        assert_eq!(graph.in_edges[&NodeId(2)].len(), 1);
        assert_eq!(graph.in_edges[&NodeId(3)].len(), 2);
        assert_eq!(graph.in_edges[&NodeId(4)].len(), 2);
    }

    #[test]
    fn test_directed_map_graph_conversion() {
        let graph = build_graph();
        // println!("{:?}", graph);
        let network_graph: DirectedNetworkGraph = graph.clone().into();

        let out_edges = [
            network_graph.out_edges(NodeId(0)).collect::<Vec<_>>(),
            network_graph.out_edges(NodeId(1)).collect::<Vec<_>>(),
            network_graph.out_edges(NodeId(2)).collect::<Vec<_>>(),
            network_graph.out_edges(NodeId(3)).collect::<Vec<_>>(),
            network_graph.out_edges(NodeId(4)).collect::<Vec<_>>(),
        ]
        .map(|s| s.into_iter().map(|s| s.1.clone()).collect::<Vec<_>>());

        let in_edges = [
            network_graph.in_edges(NodeId(0)).collect::<Vec<_>>(),
            network_graph.in_edges(NodeId(1)).collect::<Vec<_>>(),
            network_graph.in_edges(NodeId(2)).collect::<Vec<_>>(),
            network_graph.in_edges(NodeId(3)).collect::<Vec<_>>(),
            network_graph.in_edges(NodeId(4)).collect::<Vec<_>>(),
        ]
        .map(|s| s.into_iter().map(|s| s.1.clone()).collect::<Vec<_>>());

        assert_eq!(out_edges[0].len(), 3);
        assert_eq!(out_edges[1].len(), 1);
        assert_eq!(out_edges[2].len(), 1);
        assert_eq!(out_edges[3].len(), 0);
        assert_eq!(out_edges[4].len(), 2);

        assert_eq!(in_edges[0].len(), 0);
        assert_eq!(in_edges[1].len(), 2);
        assert_eq!(in_edges[2].len(), 1);
        assert_eq!(in_edges[3].len(), 2);
        assert_eq!(in_edges[4].len(), 2);

        let expected_out = [
            vec![
                NetworkEdge::new(0, NodeId(1), 1.0, EdgeDirection::Forward),
                NetworkEdge::new(1, NodeId(2), 2.0, EdgeDirection::Forward),
                NetworkEdge::new(2, NodeId(3), 3.0, EdgeDirection::Forward),
            ],
            vec![NetworkEdge::new(3, NodeId(4), 3.0, EdgeDirection::Both)],
            vec![NetworkEdge::new(5, NodeId(4), 4.0, EdgeDirection::Forward)],
            vec![],
            vec![
                NetworkEdge::new(9, NodeId(3), 4.0, EdgeDirection::Forward),
                NetworkEdge::new(10, NodeId(1), 3.0, EdgeDirection::Both),
            ],
        ];

        let expected_in = [
            vec![],
            vec![
                NetworkEdge::new(3, NodeId(4), 3.0, EdgeDirection::Both),
                NetworkEdge::new(4, NodeId(0), 1.0, EdgeDirection::Backward),
            ],
            vec![NetworkEdge::new(6, NodeId(0), 2.0, EdgeDirection::Backward)],
            vec![
                NetworkEdge::new(7, NodeId(0), 3.0, EdgeDirection::Backward),
                NetworkEdge::new(8, NodeId(4), 4.0, EdgeDirection::Backward),
            ],
            vec![
                NetworkEdge::new(10, NodeId(1), 3.0, EdgeDirection::Both),
                NetworkEdge::new(11, NodeId(2), 4.0, EdgeDirection::Backward),
            ],
        ];

        for i in 0..5 {
            assert_eq!(expected_out[i], out_edges[i], "out node: {}", i);
        }

        for i in 0..5 {
            assert_eq!(expected_in[i], in_edges[i], "in node: {}", i);
        }
    }

    #[test]
    fn test_directed_map_graph_conversion_and_back() {
        let graph = build_graph();
        // println!("{:?}", graph);
        let network_graph: DirectedNetworkGraph = graph.clone().into();
        let back_graph: DirectedMapGraph = network_graph.into();

        assert_eq!(graph, back_graph);
    }

    #[test]
    fn test_collect_edges_empty() {
        let in_edges = vec![];
        let out_edges = vec![];
        let (a, b, c) = super::collect_edges(in_edges, out_edges);

        assert_eq!(a, vec![]);
        assert_eq!(b, vec![]);
        assert_eq!(c, vec![]);
    }

    #[test]
    fn test_collect_edges_in_only() {
        let in_edges = vec![
            Edge {
                source: NodeId(1),
                target: NodeId(0),
                weight: 1.0,
            },
            Edge {
                source: NodeId(2),
                target: NodeId(0),
                weight: 1.0,
            },
            Edge {
                source: NodeId(3),
                target: NodeId(0),
                weight: 1.0,
            },
        ];
        let out_edges = vec![];
        let (a, b, c) = super::collect_edges(in_edges.clone(), out_edges);

        assert_eq!(a, vec![]);
        assert_eq!(b, vec![]);
        assert_eq!(c, in_edges);
    }

    #[test]
    fn test_collect_edges_out_only() {
        let in_edges = vec![];
        let out_edges = vec![
            Edge {
                source: NodeId(0),
                target: NodeId(1),
                weight: 1.0,
            },
            Edge {
                source: NodeId(0),
                target: NodeId(2),
                weight: 1.0,
            },
            Edge {
                source: NodeId(0),
                target: NodeId(3),
                weight: 1.0,
            },
        ];
        let (a, b, c) = super::collect_edges(in_edges, out_edges.clone());

        assert_eq!(a, out_edges);
        assert_eq!(b, vec![]);
        assert_eq!(c, vec![]);
    }

    #[test]
    fn test_collect_edges_all_overlapping() {
        let in_edges = vec![
            Edge {
                source: NodeId(1),
                target: NodeId(0),
                weight: 1.0,
            },
            Edge {
                source: NodeId(2),
                target: NodeId(0),
                weight: 1.0,
            },
            Edge {
                source: NodeId(3),
                target: NodeId(0),
                weight: 1.0,
            },
        ];
        let out_edges = vec![
            Edge {
                source: NodeId(0),
                target: NodeId(1),
                weight: 1.0,
            },
            Edge {
                source: NodeId(0),
                target: NodeId(2),
                weight: 1.0,
            },
            Edge {
                source: NodeId(0),
                target: NodeId(3),
                weight: 1.0,
            },
        ];
        let (a, b, c) = super::collect_edges(in_edges, out_edges.clone());

        assert_eq!(a, vec![]);
        assert_eq!(b, out_edges);
        assert_eq!(c, vec![]);
    }

    #[test]
    fn test_collecte_edges() {
        let in_edges = vec![
            Edge {
                source: NodeId(1),
                target: NodeId(0),
                weight: 1.0,
            },
            Edge {
                source: NodeId(2),
                target: NodeId(0),
                weight: 1.0,
            },
        ];
        let out_edges = vec![
            Edge {
                source: NodeId(0),
                target: NodeId(2),
                weight: 1.0,
            },
            Edge {
                source: NodeId(0),
                target: NodeId(3),
                weight: 1.0,
            },
        ];
        let (a, b, c) = super::collect_edges(in_edges, out_edges);

        assert_eq!(
            a,
            vec![Edge {
                source: NodeId(0),
                target: NodeId(3),
                weight: 1.0,
            }]
        );
        assert_eq!(
            b,
            vec![Edge {
                source: NodeId(0),
                target: NodeId(2),
                weight: 1.0,
            }]
        );
        assert_eq!(
            c,
            vec![Edge {
                source: NodeId(1),
                target: NodeId(0),
                weight: 1.0,
            }]
        );
    }
}
