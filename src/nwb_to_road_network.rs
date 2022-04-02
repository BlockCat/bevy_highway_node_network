use bevy_shapefile::RoadMap;
use network::{DirectedNetworkGraph, EdgeId, NetworkEdge, NetworkNode, NodeId};
use rusqlite::{
    params,
    types::{FromSql, FromSqlError},
    Connection,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoadNode {
    id: usize,
    pub road_id: usize,
}

impl NetworkNode for RoadNode {
    fn id(&self) -> network::NodeId {
        self.id.into()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RoadEdge {
    pub id: EdgeId,
    distance: f32,
    source: NodeId,
    target: NodeId,
}

impl NetworkEdge for RoadEdge {
    fn source(&self) -> network::NodeId {
        self.source
    }

    fn target(&self) -> network::NodeId {
        self.target
    }

    fn distance(&self) -> f32 {
        self.distance
    }
}

#[derive(Debug, Clone, Copy)]
enum RijRichting {
    Heen,
    Terug,
    Beide,
    Onbekend,
}

impl FromSql for RijRichting {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let rij_richting = String::column_result(value)?;
        let rij_richting = rij_richting
            .chars()
            .next()
            .ok_or_else(|| FromSqlError::InvalidType)?;
        match rij_richting {
            'H' => Ok(RijRichting::Heen),
            'T' => Ok(RijRichting::Terug),
            'B' => Ok(RijRichting::Beide),
            'O' => Ok(RijRichting::Onbekend),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

pub fn preprocess_roadmap<P: AsRef<Path>>(
    roadmap: &RoadMap,
    database: P,
) -> DirectedNetworkGraph<RoadNode, RoadEdge> {
    let database = Connection::open(database).expect("Could not open database");

    // collect edges.
    let roads = &roadmap.roads;

    let mut edges = Vec::with_capacity(roads.len());
    let mut nodes = HashMap::with_capacity(roads.len());

    let mut statement = database
        .prepare(
            "SELECT junction_id_begin, junction_id_end, rij_richting FROM wegvakken WHERE id=?",
        )
        .expect("Could not prepare statement");
    let mut out_collection: HashMap<NodeId, Vec<EdgeId>> = HashMap::new();
    let mut in_collection: HashMap<NodeId, Vec<EdgeId>> = HashMap::new();

    for (&id, section) in roads {
        let (road_id_start, road_id_end, rij_richting) = statement
            .query_row::<(usize, usize, RijRichting), _, _>(params![id], |f| {
                Ok((f.get(0)?, f.get(1)?, f.get(2)?))
            })
            .unwrap();

        let source = insert_new_node(road_id_start, &mut nodes);
        let target = insert_new_node(road_id_end, &mut nodes);

        let distance = section.points.windows(2).map(|w| w[0].distance(w[1])).sum();

        match rij_richting {
            RijRichting::Heen => insert_new_edge(
                distance,
                source,
                target,
                &mut edges,
                &mut out_collection,
                &mut in_collection,
            ),
            RijRichting::Terug => insert_new_edge(
                distance,
                target,
                source,
                &mut edges,
                &mut out_collection,
                &mut in_collection,
            ),
            RijRichting::Beide | RijRichting::Onbekend => {
                insert_new_edge(
                    distance,
                    source,
                    target,
                    &mut edges,
                    &mut out_collection,
                    &mut in_collection,
                );
                insert_new_edge(
                    distance,
                    target,
                    source,
                    &mut edges,
                    &mut out_collection,
                    &mut in_collection,
                );
            }
        }
    }

    let out_edges = nodes
        .iter()
        .map(|x| out_collection.get(&x.1.id()).cloned().unwrap_or_default())
        .collect::<Vec<_>>();
    let in_edges = nodes
        .iter()
        .map(|x| in_collection.get(&x.1.id()).cloned().unwrap_or_default())
        .collect::<Vec<_>>();

    let mut nodes = nodes.into_iter().map(|a| a.1).collect::<Vec<_>>();
    nodes.sort_by_key(|x| x.id());

    assert!(nodes.len() == in_edges.len());
    assert!(nodes.len() == out_edges.len());

    DirectedNetworkGraph {
        edges,
        nodes,
        in_edges,
        out_edges,
    }
}

fn insert_new_node(junction_id: usize, nodes: &mut HashMap<usize, RoadNode>) -> NodeId {
    let id = nodes.len();
    nodes
        .entry(junction_id)
        .or_insert_with(|| RoadNode {
            id,
            road_id: junction_id,
        })
        .id()
}

fn insert_new_edge(
    distance: f32,
    source: NodeId,
    target: NodeId,
    edges: &mut Vec<RoadEdge>,
    out_collection: &mut HashMap<NodeId, Vec<EdgeId>>,
    in_collection: &mut HashMap<NodeId, Vec<EdgeId>>,
) {
    let id = EdgeId(edges.len());
    edges.push(RoadEdge {
        id,
        distance,
        source,
        target,
    });
    out_collection.entry(source).or_default().push(id);
    in_collection.entry(target).or_default().push(id);
}
