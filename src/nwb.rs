use bevy_shapefile::RoadMap;
use network::{
    builder::{DirectedNetworkBuilder, EdgeBuilder, EdgeDirection, NodeBuilder},
    DirectedNetworkGraph, EdgeId, NetworkData, NodeId,
};
use rusqlite::{
    params,
    types::{FromSql, FromSqlError},
    Connection,
};
use serde::{Deserialize, Serialize};
use std::{path::Path, collections::HashMap};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct NWBNetworkData {
    node_junctions: Vec<usize>,
    edge_id: Vec<usize>, // for sql
}

impl NetworkData for NWBNetworkData {
    type NodeData = usize;
    type EdgeData = usize;

    fn node_data(&self, node: NodeId) -> &Self::NodeData {
        &self.node_junctions[node.0 as usize]
    }

    fn edge_data(&self, edge: EdgeId) -> &Self::EdgeData {
        &self.edge_id[edge.0 as usize]
    }

    fn with_size(node_size: usize, edge_size: usize) -> Self {
        NWBNetworkData {
            node_junctions: vec![0; node_size],
            edge_id: vec![0; edge_size],
        }
    }

    fn add_node(&mut self, node: NodeId, data: Self::NodeData) {
        self.node_junctions[node.0 as usize] = data;
    }

    fn add_edge(&mut self, edge: EdgeId, data: Self::EdgeData) {
        self.edge_id[edge.0 as usize] = data;
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct RoadNode {
    pub junction_id: usize,
}

impl NodeBuilder for RoadNode {
    type Data = usize;

    fn data(&self) -> Self::Data {
        self.junction_id
    }
}

#[derive(Debug, Clone)]
pub struct RoadEdge {
    road_id: usize,
    distance: f32,
    source: NodeId,
    target: NodeId,
    direction: EdgeDirection,
}

impl EdgeBuilder for RoadEdge {
    type Data = usize;

    fn source(&self) -> network::NodeId {
        self.source
    }

    fn target(&self) -> network::NodeId {
        self.target
    }

    fn data(&self) -> Self::Data {
        self.road_id
    }

    fn weight(&self) -> f32 {
        self.distance
    }

    fn direction(&self) -> network::builder::EdgeDirection {
        self.direction
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
            .ok_or_else(|| FromSqlError::InvalidType)?;
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

    let mut statement = database
        .prepare(
            "SELECT id,junction_id_begin, junction_id_end, rij_richting FROM wegvakken",
        )
        .expect("Could not prepare statement")
        .query_map([], |f| {
            let id: usize = f.get(0)?;
            let junction_start: usize = f.get(1)?;
            let junction_end: usize = f.get(2)?;
            let rij_richting: RijRichting = f.get(3)?;
            Ok((id, (junction_start, junction_end, rij_richting)))
        }).expect("Could not")
        .map(|x| x.unwrap())
        .collect::<HashMap<usize, (usize, usize, RijRichting)>>();
        

    for (&id, section) in roads {
        let (road_id_start, road_id_end, rij_richting) = statement[&id];

        let source = builder.add_node(RoadNode {
            junction_id: road_id_start,
        });
        let target = builder.add_node(RoadNode {
            junction_id: road_id_end,
        });

        let distance = section.points.windows(2).map(|w| w[0].distance(w[1])).sum();

        builder.add_edge(RoadEdge {
            source,
            target,
            direction: rij_richting.0,
            distance,
            road_id: id,
        });
    }

    builder.build()
}
