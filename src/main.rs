#![feature(iter_collect_into)]
#![feature(map_try_insert)]
#![feature(test)]
extern crate test;

use bevy::{prelude::*, DefaultPlugins};
use bevy_dutch_road_highway_node_network::{
    camera::{CameraConfig, CameraPlugin},
    ui::HighwayUiPlugin,
    world::{WorldConfig, WorldPlugin},
};
use bevy_polyline::PolylinePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.2)))
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(HighwayUiPlugin)
        .add_plugins(PolylinePlugin)
        // .add_plugins(LogDiagnosticsPlugin::default())
        // .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(WorldPlugin {
            config: WorldConfig {
                database_path: "data/database.db".into(),
                shapefile_path: "data/01-05-2024/Wegvakken/Wegvakken.shp".into(),
                road_map_path: "data/road_map.data".into(),
                directed_graph_path: "data/directed_graph.graph".into(),

                selected_colour: Color::GREEN,
                normal_colour: Color::WHITE,
            },
        })
        .add_plugins(CameraPlugin {
            config: CameraConfig {
                zoom_in: KeyCode::KeyQ,
                zoom_out: KeyCode::KeyE,
                zoom_factor: 0.99,
                speed: 10.0,
                left: KeyCode::KeyA,
                right: KeyCode::KeyD,
                up: KeyCode::KeyW,
                down: KeyCode::KeyS,
            },
        })
        .run();
}
