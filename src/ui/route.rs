use std::collections::HashSet;

use super::DirectedNetworkGraphContainer;
use crate::nwb::NwbEdgeIndex;
use crate::nwb::NwbGraph;
use crate::nwb::NwbNodeIndex;
use crate::world::WorldEntity;
use crate::world::WorldEntitySelectionType;
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContext;
use bevy_shapefile::RoadId;
use petgraph::algo;

pub struct RouteUIPlugin;

impl Plugin for RouteUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RouteState::default())
            .add_system(gui_system)
            .add_system(route_draw);
    }
}

#[derive(Debug, Default, Resource)]
pub struct RouteState {
    find_nodes: bool,
    node_1: Option<NwbNodeIndex>,
    node_2: Option<NwbNodeIndex>,
    edges: Option<Vec<NwbEdgeIndex>>,
}

pub fn gui_system(mut egui_context: ResMut<EguiContext>, mut state: ResMut<RouteState>) {
    egui::Window::new("Preprocessing").show(egui_context.ctx_mut(), |ui| {
        ui.label("Routing");

        if ui.button("Start route").clicked() {
            state.find_nodes = false;
            state.node_1 = None;
            state.node_2 = None;
        }

        let n1 = state
            .node_1
            .map(|x| format!("Node: {x:?}"))
            .unwrap_or("Node: none".into());
        let n2 = state
            .node_1
            .map(|x| format!("Node: {x:?}"))
            .unwrap_or("Node: none".into());

        ui.label(n1);
        ui.label(n2);
    });
}

fn route_draw(
    route_state: Res<RouteState>,
    mut query: Query<&mut WorldEntity>,
    network: Res<DirectedNetworkGraphContainer>,
) {
    if let Some(route) = &route_state.edges {
        // Collect all roadIds.
        let l = route.iter().map(|e| network[*e]).collect::<HashSet<_>>();

        query.for_each_mut(|mut a| {
            if l.contains(&a.id) {
                a.selected = WorldEntitySelectionType::Route;
            }
        });
    }
}

fn find_route<E, F>(
    source: NwbNodeIndex,
    target: NwbNodeIndex,
    graph: &NwbGraph,
    edge_cost: E,
    estimate_to_target: F,
) -> Result<Vec<NwbEdgeIndex>, String>
where
    E: Fn(&RoadId) -> f32,
    F: Fn(NwbNodeIndex, NwbNodeIndex) -> f32,
{
    let path = algo::astar(
        graph,
        source,
        |finish| finish == target,
        |e| edge_cost(e.weight()),
        |n| estimate_to_target(n, target),
    );

    path.map(|path| {
        path.1
            .windows(2)
            .map(|nodes| graph.find_edge(nodes[0], nodes[1]).unwrap())
            .collect::<Vec<_>>()
    })
    .ok_or_else(|| String::from("No path found"))
}
