use bevy::{prelude::*, DefaultPlugins};
use bevy_dutch_road_highway_node_network::{
    camera::{CameraConfig, CameraPlugin},
    ui::HighwayUiPlugin,
    world::{WorldConfig, WorldPlugin},
};
use bevy_polyline::PolylinePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::linear_rgb(0.0, 0.0, 0.2)))
        .add_plugins(DefaultPlugins)
        .add_plugins(HighwayUiPlugin)
        .add_plugins(PolylinePlugin)
        .add_plugins(WorldPlugin {
            config: WorldConfig {
                geopackage_path: "data/01-02-2026/Wegvakken/Wegvakken.gpkg".into(),
                road_map_path: "data/road_map.data".into(),
                directed_graph_path: "data/directed_graph.graph".into(),

                selected_colour: Color::linear_rgb(0.0, 1.0, 0.0),
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
