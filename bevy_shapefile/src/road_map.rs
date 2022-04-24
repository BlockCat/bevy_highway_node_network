use crate::{
    spatial::{JunctionSpatialIndex, RoadSection, RoadSpatialIndex},
    ShapeError,
};
use bevy::{
    math::{Vec2, Vec3},
    render::primitives::Aabb,
};
use rayon::prelude::*;
use rstar::{RStarInsertionStrategy, RTree, RTreeParams};
use serde::{Deserialize, Serialize};
use shapefile::{
    dbase::{FieldValue, Record},
    Polyline,
};
use std::{collections::HashMap, fs::File, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct RoadMap {
    pub roads: HashMap<usize, RoadSection>,
    pub junction_spatial: rstar::RTree<JunctionSpatialIndex, Params>,
    pub road_spatial: rstar::RTree<RoadSpatialIndex, Params>,
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
        let value = bincode::deserialize_from(file).expect("Could not deserialize");

        value
    }

    pub fn from_shapefile<P: AsRef<Path>>(path: P) -> Result<Self, ShapeError> {
        println!("Start read");

        let roads =
            shapefile::read_as::<P, Polyline, Record>(path).map_err(|x| ShapeError::Shape(x))?;

        let roads = roads.into_par_iter().collect::<Vec<_>>();

        println!("Start conversion");

        let junctions = roads
            .par_iter()
            .flat_map_iter(|(line, record)| {
                let junction_start = get_usize(record, "JTE_ID_BEG").unwrap();
                let junction_end = get_usize(record, "JTE_ID_END").unwrap();

                let start = line.part(0).and_then(|p| p.first()).unwrap();
                let end = line.part(0).and_then(|p| p.last()).unwrap();

                [
                    (junction_start, Vec2::new(start.x as f32, start.y as f32)),
                    (junction_end, Vec2::new(end.x as f32, end.y as f32)),
                ]
            })
            .collect::<HashMap<_, _>>();

        let roads = roads
            .into_par_iter()
            .enumerate()
            .map(|(id, (line, _))| {
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

                (id, RoadSection { id, points, aabb })
            })
            .collect::<HashMap<_, _>>();

        println!("Finish conversion");
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
