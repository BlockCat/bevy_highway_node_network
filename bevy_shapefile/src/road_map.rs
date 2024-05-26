use crate::{
    spatial::{JunctionSpatialIndex, RoadSection, RoadSpatialIndex},
    JunctionId, RoadId, ShapeError,
};
use bevy::{
    math::{Vec2, Vec3},
    prelude::Resource,
    render::primitives::Aabb,
};
use rayon::prelude::*;
use rstar::{RStarInsertionStrategy, RTree, RTreeParams};
use serde::{Deserialize, Serialize};
use shapefile::{
    dbase::{FieldValue, Record},
    record::polyline::GenericPolyline,
    Point, Polyline,
};
use std::{collections::HashMap, fs::File, path::Path};

/// Load shapefile.
/// This shapefile is used vor visualization of road data.
/// It loads all the road sections, and puts it in spatial data structures.
#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct RoadMap {
    pub roads: HashMap<RoadId, RoadSection>,
    pub junction_spatial: rstar::RTree<JunctionSpatialIndex, Params>,
    pub road_spatial: rstar::RTree<RoadSpatialIndex, Params>,
}

impl RoadMap {
    pub fn road_length(&self, road_id: RoadId) -> f32 {
        let section = &self.roads[&road_id];

        section
            .points
            .windows(2)
            .map(|points| points[0].distance(points[1]))
            .sum()
    }
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
    pub fn write<P: AsRef<Path>>(&self, path: P) {
        let file = File::create(path).expect("Could not create file");
        bincode::serialize_into(file, self).expect("Could not write");
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Self {
        let file = File::open(path).expect("Could not open file");
        bincode::deserialize_from(file).expect("Could not deserialize")
    }

    /// Load data from a shapefile
    pub fn from_shapefile<P: AsRef<Path>>(path: P) -> Result<Self, ShapeError> {
        println!("Start read of road data");
        let roads =
            shapefile::read_as::<P, Polyline, Record>(path).map_err(|x| ShapeError::Shape(x))?;

        println!("Loading junction data");
        let junctions = load_junctions(&roads);

        println!("Loading roads");
        let roads = load_road_sections(roads);
        println!("Finish loading roads");

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
}

fn get_usize(record: &Record, name: &str) -> Option<usize> {
    let value = record.get(name).unwrap();

    if let FieldValue::Numeric(x) = value {
        return x.map(|x| x as usize);
    }
    unreachable!();
}

/// Load junction point data
fn load_junctions(roads: &Vec<(GenericPolyline<Point>, Record)>) -> HashMap<JunctionId, Vec2> {
    roads
        .par_iter()
        .flat_map_iter(|(line, record)| {
            let junction_start = get_usize(record, "JTE_ID_BEG").unwrap();
            let junction_end = get_usize(record, "JTE_ID_END").unwrap();

            let start = line.part(0).and_then(|p| p.first()).unwrap();
            let end = line.part(0).and_then(|p| p.last()).unwrap();

            [
                (
                    JunctionId::from(junction_start),
                    Vec2::new(start.x as f32, start.y as f32),
                ),
                (
                    JunctionId::from(junction_end),
                    Vec2::new(end.x as f32, end.y as f32),
                ),
            ]
        })
        .collect::<HashMap<_, _>>()
}

/// Load road sections
fn load_road_sections(
    roads: Vec<(GenericPolyline<Point>, Record)>,
) -> HashMap<RoadId, RoadSection> {
    let roads = roads
        .into_par_iter()
        .enumerate()
        .map(|(id, (line, r))| {
            assert!(line.parts().len() == 1);

            let points = line
                .part(0)
                .expect("Could not get ?")
                .iter()
                .map(|point| Vec2::new(point.x as f32, point.y as f32))
                .collect::<Vec<_>>();

            let bbox = line.bbox();
            let aabb = Aabb::from_min_max(
                Vec3::new(bbox.x_range()[0] as f32, bbox.y_range()[0] as f32, 0.0),
                Vec3::new(bbox.x_range()[1] as f32, bbox.y_range()[1] as f32, 0.0),
            );
            let id = RoadId::from(id);

            (id, RoadSection { id, points, aabb })
        })
        .collect::<HashMap<_, _>>();
    roads
}
