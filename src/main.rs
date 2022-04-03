#![feature(iter_collect_into)]
#![feature(map_try_insert)]
use crate::camera::{CameraConfig, CameraPlugin};
use bevy::{prelude::*, DefaultPlugins};
use bevy_prototype_lyon::plugin::ShapePlugin;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{Cursor, Write},
    path::Path,
};
use world::{WorldConfig, WorldPlugin};

mod camera;
mod geo_coords;
mod nwb;
mod world;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.2)))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(WorldPlugin {
            config: WorldConfig {
                database_path: "data/database.db".into(),
                shapefile_path: "data/01-03-2022/Wegvakken/Wegvakken.shp".into(),
                road_map_path: "data/road_map.data".into(),
                directed_graph_path: "data/directed_graph.graph".into(),

                selected_colour: Color::GREEN,
                normal_colour: Color::WHITE,
            },
        })
        .add_plugin(CameraPlugin {
            config: CameraConfig {
                zoom_in: KeyCode::Q,
                zoom_out: KeyCode::E,
                zoom_factor: 0.99,
                speed: 10.0,
                left: KeyCode::A,
                right: KeyCode::D,
                up: KeyCode::W,
                down: KeyCode::S,
            },
        })
        .run();
}

pub fn write_file<T: Serialize, P: AsRef<Path>>(
    value: &T,
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;

    println!("Started writing file: {:?}", path.as_ref());

    let code = bincode::serialize(value)?;
    let result = zstd::encode_all(Cursor::new(code), 0)?;
    let mut file = File::create(&path)?;

    file.write_all(&result)?;

    println!("Finished writing file: {:?}", path.as_ref());

    Ok(())
}

pub fn read_file<T: DeserializeOwned, P: AsRef<Path>>(
    path: P,
) -> Result<T, Box<dyn std::error::Error>> {
    use std::fs::File;

    println!("Started reading file: {:?}", path.as_ref());

    let file = File::open(&path)?;

    let result = zstd::decode_all(file)?;
    let d = bincode::deserialize(&result)?;

    println!("Finished reading file: {:?}", path.as_ref());

    Ok(d)
}
