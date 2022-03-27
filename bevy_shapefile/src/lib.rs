pub use road_map::*;
use std::path::Path;

// mod road_data;
mod road_map;

pub type AABB = rstar::AABB<[f32; 2]>;

pub fn load_file<P: AsRef<Path>>(path: P) -> Result<RoadMap, ShapeError> {
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
