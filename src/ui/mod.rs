use crate::{
    nwb::NwbGraph,
    world::{WorldEntity, WorldEntitySelectionType},
};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_shapefile::RoadMap;
pub use layers::PreProcess;

use petgraph::visit::IntoNodeReferences;
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use self::layers::LayerState;

mod layers;
mod route;

pub struct HighwayUiPlugin;

#[derive(Resource, Debug)]
// pub struct DirectedNetworkGraphContainer(pub DirectedNetworkGraph<NWBNetworkData>);
pub struct DirectedNetworkGraphContainer(pub NwbGraph);

impl Deref for DirectedNetworkGraphContainer {
    type Target = NwbGraph;

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
        app.add_plugin(EguiPlugin)
            .insert_resource(LayerState {
                preprocess_layers: 6,
                neighbourhood_size: 30,
                contraction_factor: 2.0,
                base_selected: false,
                layers_selected: vec![],
                processing: false,
            })
            .add_system(layers::colouring_system)
            .add_system(layers::handle_preprocess_task)
            .add_system(layers::gui_system)
            .add_system(point_system);
    }
}

fn point_system(
    windows: Res<Windows>,
    network: Res<DirectedNetworkGraphContainer>,
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

                let node_id = network
                    .node_references()
                    .find(|n| n.1 .0 == node.junction_id)
                    .map(|n| n.0)
                    .unwrap();

                let out_edges = network
                    .edges_directed(node_id, petgraph::Direction::Outgoing)
                    .map(|x| *x.weight())
                    .collect::<HashSet<_>>();
                let in_edges = network
                    .edges_directed(node_id, petgraph::Direction::Incoming)
                    .map(|x| *x.weight())
                    .collect::<HashSet<_>>();

                query.for_each_mut(|mut we| {
                    match (out_edges.contains(&we.id), in_edges.contains(&we.id)) {
                        (true, true) => we.selected = WorldEntitySelectionType::BiDirection,
                        (true, false) => we.selected = WorldEntitySelectionType::Incoming,
                        (false, true) => we.selected = WorldEntitySelectionType::Outgoing,
                        _ => {}
                    }
                });
            }
        }
    }
}
