#![feature(iter_collect_into)]
#![feature(map_try_insert)]
use crate::camera::{CameraConfig, CameraPlugin};
use bevy::{prelude::*, DefaultPlugins};
use bevy_prototype_lyon::plugin::ShapePlugin;
use world::{WorldConfig, WorldPlugin};

mod camera;
mod geo_coords;
mod world;
mod nwb_to_road_network;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.2)))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(WorldPlugin {
            config: WorldConfig {
                database_path: "data/database.db".into(),
                data_path: "data/01-03-2022/Wegvakken/Wegvakken.shp".into(),
                compiled_path: "data/compiled.data".into(),
                network_path: "data/network.graph".into(),

                selected_colour: Color::WHITE,
                normal_colour: Color::GREEN,
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
