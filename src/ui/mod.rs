use crate::{
    nwb::NWBNetworkData,
    world::{WorldEntity, WorldEntitySelectionType},
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiPlugin;
use bevy_shapefile::RoadMap;
pub use layers::PreProcess;
use network::{DirectedNetworkGraph, NodeId};
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use self::layers::LayerState;

mod layers;
mod route;

pub struct HighwayUiPlugin;

#[derive(Resource, Debug)]
pub struct DirectedNetworkGraphContainer(pub DirectedNetworkGraph<NWBNetworkData>);

impl Deref for DirectedNetworkGraphContainer {
    type Target = DirectedNetworkGraph<NWBNetworkData>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DirectedNetworkGraphContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Plugin for HighwayUiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(EguiPlugin)
            .insert_resource(LayerState {
                preprocess_layers: 6,
                neighbourhood_size: 30,
                contraction_factor: 2.0,
                base_selected: false,
                layers_selected: vec![],
                processing: false,
            })
            .add_systems(Update, layers::colouring_system)
            .add_systems(Update, layers::handle_preprocess_task)
            .add_systems(Update, layers::gui_system)
            .add_systems(Update, mouse_point_system);
    }
}

fn mouse_point_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    network: Res<DirectedNetworkGraphContainer>,
    road_map: Res<RoadMap>,
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut query: Query<&mut WorldEntity>,
) {
    if let Ok(window) = windows.get_single() {
        if let Ok((transform, camera)) = camera_q.get_single() {
            if let Some(position) = window.cursor_position() {
                let position = Vec2::new(
                    2.0 * position.x / window.width() - 1.0,
                    -(2.0 * position.y / window.height() - 1.0),
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

                query.iter_mut().for_each(|mut we| {
                    match (out_edges.contains(&we.id), in_edges.contains(&we.id)) {
                        (true, true) => we.selected = WorldEntitySelectionType::BiDirection,
                        (true, false) => we.selected = WorldEntitySelectionType::Incoming,
                        (false, true) => we.selected = WorldEntitySelectionType::Outgoing,
                        _ => {}
                    }
                });

                return;
            }
        }
    }
}
