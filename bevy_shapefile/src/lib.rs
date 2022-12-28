pub use road_map::*;
use serde::{Deserialize, Serialize};
pub use spatial::*;
use std::path::Path;

// mod road_data;
mod road_map;
mod spatial;

pub type AABB = rstar::AABB<[f32; 2]>;

// pub const SKIP_TYPES: [&'static str; 7] = ["FP", "BU", "VP", "OVB", "CADO", "RP", "VV"];
pub const SKIP_TYPES: [&'static str; 0] = [];

pub fn from_shapefile<P: AsRef<Path>>(path: P) -> Result<RoadMap, ShapeError> {
    println!("Start loading file");

    let map = RoadMap::from_shapefile(path)?;

    println!("Finished bundling");

    Ok(map)
}

#[derive(Debug)]
pub enum ShapeError {
    IO,
    Shape(shapefile::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JunctionId(usize);

impl JunctionId {
    pub fn num(&self) -> usize {
        self.0
    }
}

impl From<usize> for JunctionId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoadId(usize);

impl RoadId {
    pub fn num(&self) -> usize {
        self.0
    }
}

impl From<usize> for RoadId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}
