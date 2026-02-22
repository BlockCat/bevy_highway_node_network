use crate::{spatial::RoadSection, JunctionId, JunctionSpatialIndex, RoadId, RoadSpatialIndex};
use bevy::{
    math::{bounding::Aabb2d, Vec2},
    prelude::Resource,
};
use rstar::{RStarInsertionStrategy, RTree, RTreeParams};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::Path,
};

/// Load shapefile.
/// This shapefile is used vor visualization of road data.
/// It loads all the road sections, and puts it in spatial data structures.
#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct RoadMap {
    roads: HashMap<RoadId, RoadSection>,
    junction_spatial: rstar::RTree<JunctionSpatialIndex, Params>,
    road_spatial: rstar::RTree<RoadSpatialIndex, Params>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Params;

impl RTreeParams for Params {
    const MIN_SIZE: usize = 2;
    const MAX_SIZE: usize = 40;
    const REINSERTION_COUNT: usize = 1;
    type DefaultInsertionStrategy = RStarInsertionStrategy;
}

impl RoadMap {
    pub fn nearest_junction(&self, x: f32, y: f32) -> Option<JunctionId> {
        self.junction_spatial
            .nearest_neighbor(&[x, y])
            .map(|x| x.junction_id)
    }

    pub fn load_roads_in_aabb2d(&self, aabb: &Aabb2d) -> HashSet<RoadId> {
        self.road_spatial
            .locate_in_envelope_intersecting(&rstar::AABB::from_corners(
                [aabb.min.x, aabb.min.y],
                [aabb.max.x, aabb.max.y],
            ))
            .map(|r| r.id)
            .collect()
    }

    pub fn get_section(&self, id: &RoadId) -> Option<&RoadSection> {
        self.roads.get(id)
    }

    pub fn roads(&self) -> &HashMap<RoadId, RoadSection> {
        &self.roads
    }

    pub fn road_length(&self, id: RoadId) -> f32 {
        self.roads
            .get(&id)
            .map(|section| section.points.windows(2).map(|w| w[0].distance(w[1])).sum())
            .unwrap_or(0.0)
    }
}

impl RoadMap {
    pub fn write<P: AsRef<Path>>(&self, path: P) {
        let file = File::create(path).expect("Could not create file");
        bincode::serialize_into(file, self).expect("Could not write");
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(path).expect("Could not open file");
        bincode::deserialize_from(file).expect("Could not deserialize")
    }

    pub fn from_geopackage<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let connection = Connection::open(path).expect("Could not open database");
        // unsafe {
        //     connection.load_extension_enable()?;
        //     connection.load_extension("mod_spatialite", None::<&str>)?;
        // }

        let (roads, junctions) = Self::load_geopackage_junctions(&connection)?;

        println!("Creating spatial data");
        let spatial_indeces = roads
            .iter()
            .map(|(id, section)| RoadSpatialIndex {
                id: *id,
                aabb: section.aabb.clone(),
            })
            .collect();

        let junction_indeces = junctions
            .into_iter()
            .map(|x| JunctionSpatialIndex {
                junction_id: x.0,
                location: x.1,
            })
            .collect::<Vec<_>>();

        println!("Inserting spatial indices");

        let road_spatial: RTree<RoadSpatialIndex, Params> =
            RTree::bulk_load_with_params(spatial_indeces);

        let junction_spatial = RTree::bulk_load_with_params(junction_indeces);

        println!("Created tree");

        Ok(RoadMap {
            roads,
            road_spatial,
            junction_spatial,
        })
    }

    fn load_geopackage_junctions(
        connection: &Connection,
    ) -> Result<(HashMap<RoadId, RoadSection>, HashMap<JunctionId, Vec2>), anyhow::Error> {
        let road_count: usize =
            connection.query_row("SELECT COUNT(*) FROM Wegvakken", [], |row| row.get(0))?;

        let mut roads = HashMap::with_capacity(road_count);
        let mut junctions = HashMap::with_capacity(road_count * 2);

        let mut statement =
            connection.prepare_cached("SELECT id, JTE_ID_BEG, JTE_ID_END, geom FROM Wegvakken")?;

        statement
            .query_map([], |row| {
                let road_id: RoadId = row.get(0)?;
                let junction_start: JunctionId = row.get(1)?;
                let junction_end: JunctionId = row.get(2)?;
                let geom: RoadSection = row.get(3)?;
                Ok((road_id, junction_start, junction_end, geom))
            })?
            .filter_map(|r| r.ok())
            .for_each(|(road_i, junction_start, junction_end, road_section)| {
                junctions.insert(junction_start, road_section.points.first().unwrap().clone());
                junctions.insert(junction_end, road_section.points.last().unwrap().clone());
                roads.insert(road_i, road_section);
            });

        Ok((roads, junctions))
    }
}
