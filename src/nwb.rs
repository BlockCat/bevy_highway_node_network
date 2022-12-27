use bevy::math::Vec2;
use bevy_shapefile::{JunctionId, RoadId, RoadMap};
use network::{
    builder::{EdgeBuilder, EdgeDirection, NodeBuilder},
    EdgeId, NetworkData, NodeId, ShortcutState,
};
use petgraph::stable_graph::{IndexType, StableDiGraph};
use rusqlite::{
    types::{FromSql, FromSqlError},
    Connection,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, path::Path};

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
pub struct NwbIndex(usize);
unsafe impl IndexType for NwbIndex {
    fn new(x: usize) -> Self {
        Self(x)
    }

    fn index(&self) -> usize {
        self.0
    }

    fn max() -> Self {
        NwbIndex(usize::MAX)
    }
}
pub type NwbGraph = StableDiGraph<(JunctionId, Vec2), RoadId, NwbIndex>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NWBNetworkData {
    pub node_junctions: Vec<(JunctionId, Vec2)>,
    edge_id: Vec<RoadId>,
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

    fn edge_road_id(&self, edge: EdgeId) -> network::ShortcutState<usize> {
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

impl Hash for JunctionNode {
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
    sql_id: RoadId, // Points to sql
    distance: f32,
    source: NodeId,
    target: NodeId,
    direction: EdgeDirection,
}

impl EdgeBuilder for RoadEdge {
    type Data = RoadId;

    fn data(&self) -> Self::Data {
        self.sql_id
    }

    fn source(&self) -> network::NodeId {
        self.source
    }

    fn target(&self) -> network::NodeId {
        self.target
    }

    fn weight(&self) -> f32 {
        self.distance
    }

    fn direction(&self) -> network::builder::EdgeDirection {
        self.direction
    }

    fn road_id(&self) -> ShortcutState<usize> {
        ShortcutState::Single(self.sql_id.num())
    }
}

#[derive(Debug, Clone, Copy)]
struct RijRichting(EdgeDirection);

impl FromSql for RijRichting {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let rij_richting = String::column_result(value)?;
        let rij_richting = rij_richting
            .chars()
            .next()
            .ok_or(FromSqlError::InvalidType)?;
        match rij_richting {
            'H' => Ok(RijRichting(EdgeDirection::Forward)),
            'T' => Ok(RijRichting(EdgeDirection::Backward)),
            'B' => Ok(RijRichting(EdgeDirection::Both)),
            'O' => Ok(RijRichting(EdgeDirection::Both)),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

pub fn preprocess_roadmap<P: AsRef<Path>>(roadmap: &RoadMap, database: P) -> NwbGraph {
    let database = Connection::open(database).expect("Could not open database");

    let mut graph = NwbGraph::default();

    let roads = &roadmap.roads;
    let statement = database
        .prepare("SELECT id,junction_id_begin, junction_id_end, rij_richting FROM wegvakken")
        .expect("Could not prepare statement")
        .query_map([], |f| {
            let id: usize = f.get(0)?;
            let junction_start: usize = f.get(1)?;
            let junction_end: usize = f.get(2)?;
            let rij_richting: RijRichting = f.get(3)?;

            let id = RoadId::from(id);
            let junction_start = JunctionId::from(junction_start);
            let junction_end = JunctionId::from(junction_end);

            Ok((id, (junction_start, junction_end, rij_richting)))
        })
        .expect("Could not")
        .map(|x| x.unwrap())
        .collect::<HashMap<RoadId, (JunctionId, JunctionId, RijRichting)>>();

    let junction_to_node = roadmap
        .junction_spatial
        .iter()
        .map(|s| {
            let junction_id = s.junction_id;

            (junction_id, graph.add_node((junction_id, s.location)))
        })
        .collect::<HashMap<_, _>>();

    for (&road_id, _) in roads {
        let (road_id_start, road_id_end, rij_richting) = statement[&road_id];

        let source = junction_to_node[&road_id_start];
        let target = junction_to_node[&road_id_end];

        match rij_richting.0 {
            EdgeDirection::Forward => {
                graph.add_edge(source, target, road_id);
            }
            EdgeDirection::Both => {
                graph.add_edge(source, target, road_id);
                graph.add_edge(target, source, road_id);
            }
            EdgeDirection::Backward => {
                graph.add_edge(target, source, road_id);
            }
        }
    }
    graph
}
