use crate::{DirectedNetworkGraph, NetworkData, NetworkEdge, NetworkNode, NodeId, ShortcutState};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EdgeDirection {
    Forward,
    Both,
    Backward,
}

pub trait NodeBuilder: Hash + PartialEq + Eq {
    type Data: Clone;

    fn data(&self) -> Self::Data;
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DefaultNodeBuilder(pub NodeId);

impl NodeBuilder for DefaultNodeBuilder {
    type Data = ();

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
    pub fn backward(
        source: NodeId,
        target: NodeId,
        road_id: usize,
        weight: f32,
    ) -> DefaultEdgeBuilder {
        Self(source, target, weight, road_id, EdgeDirection::Backward)
    }
    pub fn both(source: NodeId, target: NodeId, road_id: usize, weight: f32) -> DefaultEdgeBuilder {
        Self(source, target, weight, road_id, EdgeDirection::Both)
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

        let mut map = HashMap::<NodeId, HashMap<NodeId, &E>>::new();

        for (_, edge) in &self.edges {
            map.entry(edge.source())
                .or_default()
                .insert(edge.target(), edge);
            map.entry(edge.target())
                .or_default()
                .insert(edge.source(), edge);
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
    map: &HashMap<NodeId, HashMap<NodeId, &E>>,
    node_id: NodeId,
) -> Vec<(NodeId, EdgeDirection, NodeId, E)> {
    let mut build_edges = map[&node_id]
        .iter()
        .map(|(target, edge)| {
            let direction = if map
                .get(target)
                .and_then(|s| s.get(&node_id))
                .filter(|x| x.weight() == edge.weight())
                .is_some()
            {
                EdgeDirection::Both
            } else {
                if edge.source() == node_id {
                    EdgeDirection::Forward
                } else {
                    EdgeDirection::Backward
                }
            };

            (node_id, direction, *target, (*edge).clone())
        })
        .collect::<Vec<_>>();
    build_edges.sort_by_key(|x| (x.0, x.1, x.2));
    build_edges
}

#[cfg(test)]
mod tests {
    use crate::{
        tests::{TestEdge, TestNode},
        DirectedNetworkGraph,
    };

    fn create_network() -> DirectedNetworkGraph<()> {
        let mut builder = crate::builder::DirectedNetworkBuilder::new();

        let na = builder.add_node(TestNode(0));
        let nb = builder.add_node(TestNode(1));
        let nc = builder.add_node(TestNode(2));
        let nd = builder.add_node(TestNode(3));
        let ne = builder.add_node(TestNode(4));
        let nf = builder.add_node(TestNode(5));

        builder.add_edge(TestEdge::forward(na, nb, 10.0));
        builder.add_edge(TestEdge::forward(nb, nd, 12.0));
        builder.add_edge(TestEdge::forward(nd, ne, 2.0));
        builder.add_edge(TestEdge::forward(na, nc, 15.0));
        builder.add_edge(TestEdge::forward(nc, ne, 10.0));
        builder.add_edge(TestEdge::forward(nf, ne, 5.0));
        builder.add_edge(TestEdge::forward(nb, nf, 15.0));
        builder.add_edge(TestEdge::forward(nd, nf, 1.0));

        builder.build()
    }

    #[test]
    fn builder_test() {
        let n1 = crate::tests::create_ref_network_1();
        let n2 = create_network();

        assert_eq!(n1, n2);
    }
}
