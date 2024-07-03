use crate::{DirectedNetworkGraph, NetworkData, NetworkEdge, NetworkNode, NodeId, ShortcutState};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, hash::Hash, ops::Neg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EdgeDirection {
    Forward,
    Both,
    Backward,
}

impl Neg for EdgeDirection {
    type Output = EdgeDirection;

    fn neg(self) -> Self::Output {
        match self {
            EdgeDirection::Forward => EdgeDirection::Backward,
            EdgeDirection::Both => EdgeDirection::Both,
            EdgeDirection::Backward => EdgeDirection::Forward,
        }
    }
}

pub trait NodeBuilder: Hash + PartialEq + Eq {
    type Data: Clone;

    fn id(&self) -> u32;
    fn data(&self) -> Self::Data;
}

impl NodeBuilder for usize {
    type Data = ();

    fn id(&self) -> u32 {
        *self as u32
    }
    fn data(&self) -> Self::Data {
        ()
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DefaultNodeBuilder(pub NodeId);

impl NodeBuilder for DefaultNodeBuilder {
    type Data = ();

    fn id(&self) -> u32 {
        self.0 .0
    }

    fn data(&self) -> Self::Data {
        ()
    }
}

pub trait EdgeBuilder: Clone {
    type Data: Clone;
    fn data(&self) -> Self::Data;
    fn road_id(&self) -> ShortcutState<usize>;
    fn source(&self) -> NodeId;
    fn target(&self) -> NodeId;
    fn weight(&self) -> f32;
    fn direction(&self) -> EdgeDirection;
}

#[derive(Debug, Clone)]
pub struct DefaultEdgeBuilder(NodeId, NodeId, f32, usize, EdgeDirection);

impl EdgeBuilder for DefaultEdgeBuilder {
    type Data = ();

    fn data(&self) -> Self::Data {
        ()
    }

    fn source(&self) -> NodeId {
        self.0
    }

    fn target(&self) -> NodeId {
        self.1
    }

    fn weight(&self) -> f32 {
        self.2
    }

    fn road_id(&self) -> ShortcutState<usize> {
        ShortcutState::Single(self.3)
    }

    fn direction(&self) -> EdgeDirection {
        self.4
    }
}

impl DefaultEdgeBuilder {
    pub fn forward(
        source: NodeId,
        target: NodeId,
        road_id: usize,
        weight: f32,
    ) -> DefaultEdgeBuilder {
        Self(source, target, weight, road_id, EdgeDirection::Forward)
    }
}

#[derive(Debug)]
pub struct DirectedNetworkBuilder<V: NodeBuilder, E: EdgeBuilder> {
    nodes: HashMap<V, NodeId>,
    edges: HashMap<(NodeId, NodeId), E>,
}

impl<V: NodeBuilder, E: EdgeBuilder> DirectedNetworkBuilder<V, E> {
    pub fn new() -> Self {
        DirectedNetworkBuilder {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }
    pub fn add_edge(&mut self, edge: E) -> &mut Self {
        self.edges.insert((edge.source(), edge.target()), edge);
        self
    }

    pub fn add_node(&mut self, node: V) -> NodeId {
        let id = self.nodes.len();
        *self.nodes.entry(node).or_insert(id.into())
    }

    pub fn build<D>(mut self) -> DirectedNetworkGraph<D>
    where
        D: NetworkData<NodeData = V::Data, EdgeData = E::Data>,
    {
        self.edges.shrink_to_fit();
        self.nodes.shrink_to_fit();

        let mut build_nodes = self.nodes.into_iter().collect::<Vec<_>>();
        build_nodes.sort_by_key(|d| d.1);

        let mut map = HashMap::<NodeId, Vec<&E>>::new();

        for (_, edge) in &self.edges {
            map.entry(edge.source()).or_default().push(edge);
            map.entry(edge.target()).or_default().push(edge);
        }

        let mut network_data = D::with_size(build_nodes.len(), self.edges.len() * 2);
        let mut nodes = Vec::with_capacity(build_nodes.len());
        let mut edges = Vec::with_capacity(self.edges.len() * 2);

        for (node_id, (node, _)) in build_nodes.iter().enumerate() {
            let node_id = NodeId::from(node_id);

            network_data.add_node(node_id, node.data());

            let start_edge_index = edges.len() as u32;

            for (_, direction, target_node, data) in collect_edges(&map, node_id) {
                let network_edge =
                    NetworkEdge::new(edges.len() as u32, target_node, data.weight(), direction);

                network_data.add_edge(network_edge.edge_id.into(), data.data(), data.road_id());
                edges.push(network_edge);
            }

            let last_edge_index = edges.len() as u32;

            let network_node = NetworkNode::new(start_edge_index, last_edge_index);

            nodes.push(network_node);
        }

        DirectedNetworkGraph::new(nodes, edges, network_data)
    }
}

fn collect_edges<E: EdgeBuilder + Sized>(
    map: &HashMap<NodeId, Vec<&E>>,
    node_id: NodeId,
) -> Vec<(NodeId, EdgeDirection, NodeId, E)> {
    let mut build_edges = map[&node_id]
        .iter()
        .map(|&edge| {
            let node_is_source = edge.source() == node_id;

            let (direction, target) = if edge.direction() == EdgeDirection::Both {
                if node_is_source {
                    (EdgeDirection::Both, edge.target())
                } else {
                    (EdgeDirection::Both, edge.source())
                }
            } else if node_is_source {
                (EdgeDirection::Forward, edge.target())
            } else {
                debug_assert_eq!(node_id, edge.target());
                (EdgeDirection::Backward, edge.source())
            };

            (node_id, direction, target, (*edge).clone())
        })
        .collect::<Vec<_>>();
    build_edges.sort_by_key(|x| (x.0, x.1, x.2));
    build_edges
}
