use bevy::math::Vec2;
use bevy_shapefile::{JunctionId, RoadId};
use graph::{
    builder::{EdgeBuilder, EdgeDirection, NodeBuilder},
    EdgeId, NetworkData, NodeId, ShortcutState,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NWBNetworkData {
    pub node_junctions: Vec<(JunctionId, Vec2)>,
    edge_id: Vec<RoadId>, // for sql, not nwb road_id
}

impl NetworkData for NWBNetworkData {
    type NodeData = (JunctionId, Vec2);
    type EdgeData = RoadId;

    fn node_data(&self, node: NodeId) -> &Self::NodeData {
        &self.node_junctions[node.0 as usize]
    }

    fn edge_data(&self, edge: EdgeId) -> &Self::EdgeData {
        &self.edge_id[edge.0 as usize]
    }

    fn with_size(node_size: usize, edge_size: usize) -> Self {
        NWBNetworkData {
            node_junctions: vec![(0.into(), Vec2::ZERO); node_size],
            edge_id: vec![0.into(); edge_size],
        }
    }

    fn add_node(&mut self, node: NodeId, data: Self::NodeData) {
        self.node_junctions[node.0 as usize] = data;
    }

    fn add_edge(&mut self, edge: EdgeId, data: Self::EdgeData, _: ShortcutState<usize>) {
        self.edge_id[edge.0 as usize] = data;
    }

    fn edge_road_id(&self, edge: EdgeId) -> graph::ShortcutState<usize> {
        ShortcutState::Single(self.edge_id[edge.0 as usize].num())
    }
}

#[derive(Debug)]
pub struct JunctionNode {
    pub junction_id: JunctionId,
    pub location: Vec2,
}

impl PartialEq for JunctionNode {
    fn eq(&self, other: &Self) -> bool {
        self.junction_id == other.junction_id
    }
}

impl Eq for JunctionNode {}

impl std::hash::Hash for JunctionNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.junction_id.hash(state);
    }
}

impl NodeBuilder for JunctionNode {
    type Data = (JunctionId, Vec2);

    fn data(&self) -> Self::Data {
        (self.junction_id, self.location)
    }

    fn id(&self) -> u32 {
        self.junction_id.num() as u32
    }
}

#[derive(Debug, Clone)]
pub struct RoadEdge {
    pub(super) sql_id: RoadId, // Points to sql
    pub(super) distance: f32,
    pub(super) source: NodeId,
    pub(super) target: NodeId,
    pub(super) direction: EdgeDirection,
}

impl EdgeBuilder for RoadEdge {
    type Data = RoadId;

    fn data(&self) -> Self::Data {
        self.sql_id
    }

    fn source(&self) -> graph::NodeId {
        self.source
    }

    fn target(&self) -> graph::NodeId {
        self.target
    }

    fn weight(&self) -> f32 {
        self.distance
    }

    fn direction(&self) -> graph::builder::EdgeDirection {
        self.direction
    }

    fn road_id(&self) -> ShortcutState<usize> {
        ShortcutState::Single(self.sql_id.num())
    }
}
