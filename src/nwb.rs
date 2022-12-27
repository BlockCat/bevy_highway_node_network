use bevy::math::Vec2;
use bevy_shapefile::{JunctionId, RoadId, RoadMap};
use network::builder::EdgeDirection;
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
