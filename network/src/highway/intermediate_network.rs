use crate::{
    builder::{DirectedNetworkBuilder, EdgeBuilder, EdgeDirection, NodeBuilder},
    DirectedNetworkGraph, EdgeId, NetworkData, NodeId, ShortcutState,
};
use rayon::iter::{FromParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IntermediateNode(pub NodeId);

#[derive(Debug, Clone)]
pub struct IntermediateEdge {
    data: ShortcutState<u32>,
    road_id: ShortcutState<usize>,
    direction: EdgeDirection,
    source: NodeId,
    target: NodeId,
    weight: f32,
}

impl NodeBuilder for IntermediateNode {
    type Data = NodeId;

    fn data(&self) -> Self::Data {
        self.0
    }
}

impl EdgeBuilder for IntermediateEdge {
    type Data = ShortcutState<u32>; // Points to road_id

    fn data(&self) -> Self::Data {
        self.data.clone()
    }

    fn source(&self) -> NodeId {
        self.source
    }

    fn target(&self) -> NodeId {
        self.target
    }

    fn weight(&self) -> f32 {
        self.weight
    }

    fn direction(&self) -> EdgeDirection {
        self.direction
    }

    fn road_id(&self) -> ShortcutState<usize> {
        self.road_id.clone()
    }
}

impl IntermediateEdge {
    pub fn new(
        source: NodeId,
        target: NodeId,
        weight: f32,
        data: ShortcutState<u32>,
        road_id: ShortcutState<usize>,
        direction: EdgeDirection,
    ) -> Self {
        Self {
            source,
            target,
            weight,
            data,
            direction,
            road_id,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct IntermediateNetwork {
    out_edges: HashMap<NodeId, HashMap<NodeId, IntermediateEdge>>,
    in_edges: HashMap<NodeId, HashMap<NodeId, IntermediateEdge>>,
}

impl IntermediateNetwork {
    pub fn nodes(&self) -> Vec<NodeId> {
        self.out_edges.keys().cloned().collect::<Vec<_>>()
    }
    pub fn out_edges(&self, node: NodeId) -> Option<&HashMap<NodeId, IntermediateEdge>> {
        self.out_edges.get(&node)
    }

    pub fn in_edges(&self, node: NodeId) -> Option<&HashMap<NodeId, IntermediateEdge>> {
        self.in_edges.get(&node)
    }
}
impl IntermediateNetwork {
    pub fn add_edge(&mut self, edge: IntermediateEdge) {
        let source = edge.source;
        let target = edge.target;
        self.out_edges
            .entry(source)
            .or_default()
            .insert(target, edge.clone());

        self.in_edges
            .entry(target)
            .or_default()
            .insert(source, edge.clone());
    }

    pub fn remove_node(&mut self, node: NodeId) {
        if let Some(outs) = self.out_edges.get(&node) {
            for edge in outs.values() {
                self.in_edges.entry(edge.target).and_modify(|x| {
                    x.remove(&node);
                });
            }
        }

        if let Some(ins) = self.in_edges.get(&node) {
            for edge in ins.values() {
                self.out_edges.entry(edge.source).and_modify(|x| {
                    x.remove(&node);
                });
            }
        }

        self.out_edges.remove(&node);
        self.in_edges.remove(&node);
    }

    pub fn bypass(&mut self, node: NodeId) -> Vec<NodeId> {
        let parents = if let Some(parents) = self.in_edges(node) {
            parents
        } else {
            self.remove_node(node);
            return vec![];
        };
        let children = if let Some(children) = self.out_edges(node) {
            children
        } else {
            self.remove_node(node);
            return vec![];
        };

        let mut collects = Vec::new();

        for (parent, parent_edge) in parents {
            debug_assert_eq!(parent, &parent_edge.source);
            for (child, child_edge) in children {
                debug_assert_eq!(child, &child_edge.target);
                if parent.0 != child.0 {
                    let distance = parent_edge.weight + child_edge.weight;
                    let (state, road_ids) = collect_shortcut_data_edges(parent_edge, child_edge);
                    let shortcut = IntermediateEdge::new(
                        *parent,
                        *child,
                        distance,
                        ShortcutState::Shortcut(state),
                        ShortcutState::Shortcut(road_ids),
                        EdgeDirection::Forward,
                    );

                    collects.push(shortcut);
                }
            }
        }

        let touched = parents.keys().chain(children.keys()).cloned().collect();

        self.remove_node(node);

        for shortcut in collects {
            self.add_edge(shortcut)
        }

        touched
    }
}

fn collect_shortcut_data_edges(
    parent_edge: &IntermediateEdge,
    child_edge: &IntermediateEdge,
) -> (Vec<u32>, Vec<usize>) {
    let mut edge_id_a = Vec::from(parent_edge.data.clone());
    let mut road_id_a = Vec::from(parent_edge.road_id.clone());

    edge_id_a.extend(Vec::from(child_edge.data.clone()));
    road_id_a.extend(Vec::from(child_edge.road_id.clone()));

    (edge_id_a, road_id_a)
}

impl FromIterator<IntermediateEdge> for IntermediateNetwork {
    fn from_iter<T: IntoIterator<Item = IntermediateEdge>>(iter: T) -> Self {
        let mut intermediate = IntermediateNetwork::default();
        for edge in iter.into_iter() {
            intermediate.add_edge(edge);
        }

        intermediate
    }
}

impl FromParallelIterator<IntermediateEdge> for IntermediateNetwork {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: rayon::iter::IntoParallelIterator<Item = IntermediateEdge>,
    {
        let mut intermediate = IntermediateNetwork::default();
        let edges = par_iter.into_par_iter().collect::<Vec<_>>();
        println!("Collected");
        for edge in edges {
            intermediate.add_edge(edge);
        }

        intermediate
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IntermediateData {
    references: HashMap<NodeId, NodeId>,
    shortcuts: HashMap<EdgeId, ShortcutState<u32>>,
    road_ids: HashMap<EdgeId, ShortcutState<usize>>,
}

impl NetworkData for IntermediateData {
    type NodeData = NodeId;
    type EdgeData = ShortcutState<u32>;

    fn node_data(&self, node: NodeId) -> &Self::NodeData {
        &self.references[&node]
    }

    fn edge_data(&self, edge: EdgeId) -> &Self::EdgeData {
        &self.shortcuts[&edge]
    }

    fn with_size(node_size: usize, edge_size: usize) -> Self {
        Self {
            references: HashMap::with_capacity(node_size),
            shortcuts: HashMap::with_capacity(edge_size),
            road_ids: HashMap::with_capacity(edge_size),
        }
    }

    fn add_node(&mut self, _: NodeId, _: Self::NodeData) {}

    fn add_edge(&mut self, edge: EdgeId, data: Self::EdgeData, road_id: ShortcutState<usize>) {
        self.shortcuts.insert(edge, data);
        self.road_ids.insert(edge, road_id);
    }

    fn edge_road_id(&self, edge: EdgeId) -> ShortcutState<usize> {
        self.road_ids[&edge].clone()
    }
}

impl From<IntermediateNetwork> for DirectedNetworkGraph<IntermediateData> {
    fn from(val: IntermediateNetwork) -> Self {
        let mut builder = DirectedNetworkBuilder::<IntermediateNode, IntermediateEdge>::new();

        for node in val.nodes() {
            for (_, edge) in val.out_edges(node).unwrap() {
                let n1 = builder.add_node(IntermediateNode(edge.source));
                let n2 = builder.add_node(IntermediateNode(edge.target));

                builder.add_edge(IntermediateEdge {
                    source: n1,
                    target: n2,
                    ..edge.clone()
                });
            }
        }

        builder.build()
    }
}
