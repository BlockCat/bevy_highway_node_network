use crate::{nwb::NWBNetworkData, world::WorldEntity};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_shapefile::RoadMap;
pub use layers::PreProcess;
use network::{DirectedNetworkGraph, NodeId};
use std::collections::HashSet;

use self::layers::LayerState;

mod layers;
mod route;

pub struct HighwayUiPlugin;

impl Plugin for HighwayUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(LayerState {
                preprocess_layers: 6,
                neighbourhood_size: 30,
                contraction_factor: 2.0,
                base_selected: false,
                layers_selected: vec![],
                processing: false
            })
            .add_system(layers::colouring_system)
            .add_system(layers::handle_preprocess_task)
            .add_system(layers::gui_system)
            .add_system(point_system);
    }
}

fn point_system(
    windows: Res<Windows>,
    network: Res<DirectedNetworkGraph<NWBNetworkData>>,
    road_map: Res<RoadMap>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut query: Query<&mut WorldEntity>,
) {
    if let Some(window) = windows.get_primary() {
        if let Ok((transform, camera)) = camera_q.get_single() {
            if let Some(position) = window.cursor_position() {
                let position = Vec2::new(
                    2.0 * position.x / window.width() - 1.0,
                    2.0 * position.y / window.height() - 1.0,
                );
                let world = crate::world::convert(position, transform, camera);
                let node = road_map
                    .junction_spatial
                    .nearest_neighbor(&[world.x, world.y])
                    .unwrap();

                let node_id = (0..network.nodes().len())
                    .map(NodeId::from)
                    .find(|x| network.node_data(*x).0 == node.junction_id)
                    .unwrap();

                let out_edges = network
                    .out_edges(node_id)
                    .map(|(id, _)| *network.edge_data(id))
                    .collect::<HashSet<_>>();

                let in_edges = network
                    .in_edges(node_id)
                    .map(|(id, _)| *network.edge_data(id))
                    .collect::<HashSet<_>>();

                query.for_each_mut(|mut we| {
                    match (
                        out_edges.contains(&(we.id as usize)),
                        in_edges.contains(&(we.id as usize)),
                    ) {
                        (true, true) => we.selected = Some(Color::GREEN),
                        (true, false) => we.selected = Some(Color::RED),
                        (false, true) => we.selected = Some(Color::YELLOW),
                        _ => {}
                    }
                });

                return;
            }
        }
    }
}
