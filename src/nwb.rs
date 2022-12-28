use bevy::math::Vec2;
use bevy_shapefile::{JunctionId, RoadId, RoadMap};
use network::{HighwayEdgeIndex, HighwayGraph, HighwayNodeIndex};
use rusqlite::{
    types::{FromSql, FromSqlError},
    Connection,
};
use std::{collections::HashMap, path::Path};

pub type NwbGraph = HighwayGraph<(JunctionId, Vec2), RoadId>;
pub type NwbNodeIndex = HighwayNodeIndex;
pub type NwbEdgeIndex = HighwayEdgeIndex;

pub fn preprocess_roadmap<P: AsRef<Path>>(roadmap: &RoadMap, database: P) -> NwbGraph {
    let database = Connection::open(database).expect("Could not open database");

    let mut graph = NwbGraph::default();

    let roads = &roadmap.roads;
    let statement = database
        .prepare(include_str!("selection.sql"))
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

    for (road_id, (road_id_start, road_id_end, rij_richting)) in roads
        .keys()
        .filter_map(|road_id| statement.get(road_id).map(|a| (*road_id, a)))
    {
        if road_id_start == road_id_end {
            println!("Found a road that starts and ends at the same place");
            continue;
        }

        let source = junction_to_node[&road_id_start];
        let target = junction_to_node[&road_id_end];

        match rij_richting {
            RijRichting::Forward => {
                graph.add_edge(source, target, road_id);
            }
            RijRichting::Both => {
                graph.add_edge(source, target, road_id);
                graph.add_edge(target, source, road_id);
            }
            RijRichting::Backward => {
                graph.add_edge(target, source, road_id);
            }
        }
    }
    graph
}

#[derive(Debug, Clone, Copy)]
enum RijRichting {
    Forward,
    Backward,
    Both,
}

impl FromSql for RijRichting {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let rij_richting = String::column_result(value)?;
        let rij_richting = rij_richting
            .chars()
            .next()
            .ok_or(FromSqlError::InvalidType)?;
        match rij_richting {
            'H' => Ok(RijRichting::Forward),
            'T' => Ok(RijRichting::Backward),
            'B' => Ok(RijRichting::Both),
            'O' => Ok(RijRichting::Both),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}
