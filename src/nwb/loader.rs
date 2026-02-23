use bevy_shapefile::{JunctionId, RoadId, RoadMap};
use graph::{
    builder::{DirectedNetworkBuilder, EdgeDirection},
    DirectedNetworkGraph,
};
use rusqlite::{
    types::{FromSql, FromSqlError},
    Connection,
};
use std::{collections::HashMap, path::Path};

use super::data::{JunctionNode, NWBNetworkData, RoadEdge};

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

    let mut builder: DirectedNetworkBuilder<JunctionNode, RoadEdge> =
        DirectedNetworkBuilder::new();

    let statement = database
        .prepare("SELECT id,JTE_ID_BEG, JTE_ID_END, RIJRICHTNG FROM wegvakken")
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

    for (road_id, section) in roadmap.roads() {
        let (road_id_start, road_id_end, rij_richting) = statement[&road_id];

        let source = builder.add_node(JunctionNode {
            junction_id: road_id_start,
            location: *section.points.first().unwrap(),
        });
        let target = builder.add_node(JunctionNode {
            junction_id: road_id_end,
            location: *section.points.last().unwrap(),
        });

        let distance = section.points.windows(2).map(|w| w[0].distance(w[1])).sum();

        builder.add_edge(RoadEdge {
            source,
            target,
            direction: rij_richting.0,
            distance,
            sql_id: *road_id,
        });
    }

    builder.build()
}
