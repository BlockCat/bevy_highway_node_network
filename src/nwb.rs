use bevy::math::Vec2;
use bevy_shapefile::RoadMap;
use network::{
    builder::{DirectedNetworkBuilder, EdgeBuilder, EdgeDirection, NodeBuilder},
    DirectedNetworkGraph, EdgeId, NetworkData, NodeId, ShortcutState,
};
use rusqlite::{
    types::{FromSql, FromSqlError},
    Connection,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, path::Path};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NWBNetworkData {
    pub node_junctions: Vec<(usize, Vec2)>,
    edge_id: Vec<usize>, // for sql, not nwb road_id
}

impl NetworkData for NWBNetworkData {
    type NodeData = (usize, Vec2);
    type EdgeData = usize;

    fn node_data(&self, node: NodeId) -> &Self::NodeData {
        &self.node_junctions[node.0 as usize]
    }

    fn edge_data(&self, edge: EdgeId) -> &Self::EdgeData {
        &self.edge_id[edge.0 as usize]
    }

    fn with_size(node_size: usize, edge_size: usize) -> Self {
        NWBNetworkData {
            node_junctions: vec![(0, Vec2::ZERO); node_size],
            edge_id: vec![0; edge_size],
        }
    }

    fn add_node(&mut self, node: NodeId, data: Self::NodeData) {
        self.node_junctions[node.0 as usize] = data;
    }

    fn add_edge(&mut self, edge: EdgeId, data: Self::EdgeData, _: ShortcutState<usize>) {
        self.edge_id[edge.0 as usize] = data;
    }

    fn edge_road_id(&self, edge: EdgeId) -> network::ShortcutState<usize> {
        ShortcutState::Single(self.edge_id[edge.0 as usize])
    }
}

#[derive(Debug)]
pub struct RoadNode {
    pub junction_id: usize,
    pub location: Vec2,
}

impl PartialEq for RoadNode {
    fn eq(&self, other: &Self) -> bool {
        self.junction_id == other.junction_id
    }
}

impl Eq for RoadNode {}

impl Hash for RoadNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.junction_id.hash(state);
    }
}

impl NodeBuilder for RoadNode {
    type Data = (usize, Vec2);

    fn data(&self) -> Self::Data {
        (self.junction_id, self.location.clone())
    }

    fn id(&self) -> u32 {
        self.junction_id as u32
    }
}

#[derive(Debug, Clone)]
pub struct RoadEdge {
    sql_id: usize, // Points to sql
    distance: f32,
    source: NodeId,
    target: NodeId,
    direction: EdgeDirection,
}

impl EdgeBuilder for RoadEdge {
    type Data = usize;

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
        ShortcutState::Single(self.sql_id)
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

pub fn preprocess_roadmap<P: AsRef<Path>>(
    roadmap: &RoadMap,
    database: P,
) -> DirectedNetworkGraph<NWBNetworkData> {
    let database = Connection::open(database).expect("Could not open database");

    let mut builder: DirectedNetworkBuilder<RoadNode, RoadEdge> = DirectedNetworkBuilder::new();
    let roads = &roadmap.roads;

    let statement = database
        .prepare("SELECT id,junction_id_begin, junction_id_end, rij_richting FROM wegvakken")
        .expect("Could not prepare statement")
        .query_map([], |f| {
            let id: usize = f.get(0)?;
            let junction_start: usize = f.get(1)?;
            let junction_end: usize = f.get(2)?;
            let rij_richting: RijRichting = f.get(3)?;
            Ok((id, (junction_start, junction_end, rij_richting)))
        })
        .expect("Could not")
        .map(|x| x.unwrap())
        .collect::<HashMap<usize, (usize, usize, RijRichting)>>();

    for (&sql_id, section) in roads {
        let (road_id_start, road_id_end, rij_richting) = statement[&sql_id];

        let source = builder.add_node(RoadNode {
            junction_id: road_id_start,
            location: section.points.first().unwrap().clone(),
        });
        let target = builder.add_node(RoadNode {
            junction_id: road_id_end,
            location: section.points.last().unwrap().clone(),
        });

        let distance = section.points.windows(2).map(|w| w[0].distance(w[1])).sum();

        builder.add_edge(RoadEdge {
            source,
            target,
            direction: rij_richting.0,
            distance,
            sql_id,
        });
    }

    builder.build()
}
