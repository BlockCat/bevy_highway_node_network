use crate::camera::{CameraConfig, CameraPlugin};
use bevy::{prelude::*, DefaultPlugins};
use bevy_prototype_lyon::plugin::ShapePlugin;
use world::{WorldConfig, WorldPlugin};

mod camera;
mod geo_coords;
mod world;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.278, 0.458, 0.901)))
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(WorldPlugin {
            config: WorldConfig {
                data_path: "data/01-03-2022/Wegvakken/Wegvakken.shp".into(),
                compiled_path: "data/compiled.data".into(),

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
