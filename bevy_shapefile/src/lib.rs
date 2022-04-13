pub use road_map::*;
pub use spatial::*;
use std::path::Path;

// mod road_data;
mod road_map;
mod spatial;

pub type AABB = rstar::AABB<[f32; 2]>;

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
