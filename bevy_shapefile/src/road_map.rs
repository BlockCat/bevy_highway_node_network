use crate::ShapeError;
use bevy::{
    math::{Vec2, Vec3},
    render::primitives::Aabb,
};
use rayon::prelude::*;
use rstar::{RStarInsertionStrategy, RTree, RTreeObject, RTreeParams};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use shapefile::Polyline;
use std::{collections::HashMap, fs::File, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct RoadMap {
    pub roads: HashMap<usize, RoadSection>,
    pub map: rstar::RTree<RoadSpatialIndex, Params>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoadSection {
    pub id: usize,
    pub points: Vec<Vec2>,
    #[serde(
        serialize_with = "serialize_aabb",
        deserialize_with = "deserialize_aabb"
    )]
    pub aabb: Aabb,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Params;

impl RTreeParams for Params {
    const MIN_SIZE: usize = 2;
    const MAX_SIZE: usize = 40;
    const REINSERTION_COUNT: usize = 1;
    type DefaultInsertionStrategy = RStarInsertionStrategy;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoadSpatialIndex {
    pub id: usize,
    #[serde(
        serialize_with = "serialize_aabb",
        deserialize_with = "deserialize_aabb"
    )]
    pub aabb: Aabb,
}

impl RTreeObject for RoadSpatialIndex {
    type Envelope = rstar::AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let min = self.aabb.min();
        let max = self.aabb.max();
        rstar::AABB::from_corners([min.x, min.y], [max.x, max.y])
    }
}

fn deserialize_aabb<'de, D>(deserializer: D) -> Result<Aabb, D::Error>
where
    D: Deserializer<'de>,
{
    let (min, max): (Vec3, Vec3) = Deserialize::deserialize(deserializer)?;

    Ok(Aabb::from_min_max(min, max))
}

fn serialize_aabb<S>(aabb: &Aabb, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let min = aabb.min();
    let max = aabb.max();

    (min, max).serialize(serializer)
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
            shapefile::read_shapes_as::<P, Polyline>(path).map_err(|x| ShapeError::Shape(x))?;

        println!("Start conversion");
        let roads = roads
            .into_par_iter()
            .enumerate()
            .map(|(id, line)| {
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

        println!("Inserting spatial indices");

        let map: RTree<RoadSpatialIndex, Params> = RTree::bulk_load_with_params(spatial_indeces);

        println!("Created tree");

        Ok(RoadMap { roads, map })
    }
}
