use super::DirectedNetworkGraphContainer;
use super::PointClickedEvent;
use crate::nwb::NwbEdgeIndex;
use crate::nwb::NwbGraph;
use crate::nwb::NwbNodeIndex;
use crate::world::WorldEntity;
use crate::world::WorldEntitySelectionType;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::tasks::Task;
use bevy_egui::egui;
use bevy_egui::EguiContext;
use bevy_shapefile::RoadId;
use bevy_shapefile::RoadMap;
use futures_lite::future;
use petgraph::algo;
use std::collections::HashSet;

pub struct RouteUIPlugin;

impl Plugin for RouteUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NodeSelectionState::default())
            .add_system(gui_system)
            .add_system(waiting_for_task)
            .add_system(route_draw);
    }
}

#[derive(Debug, Default, Resource)]
pub enum NodeSelectionState {
    #[default]
    NoRoute,
    FindingNode1,
    FindingNode2(NwbNodeIndex),
    FindingRoute(Task<Result<Vec<NwbEdgeIndex>, String>>),
    FoundRoute(Vec<NwbEdgeIndex>),
}

pub fn gui_system(
    graph: Res<DirectedNetworkGraphContainer>,
    road_map: Res<RoadMap>,
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<NodeSelectionState>,
    mut event_reader: EventReader<PointClickedEvent>,
) {
    
    egui::Window::new("Routing").show(egui_context.ctx_mut(), |ui| {
        ui.label("Routing");

        if ui.button("Start route").clicked() {
            *state = NodeSelectionState::FindingNode1;
            event_reader.clear();
        } else {
            match (state.as_ref(), event_reader.iter().next()) {
                (NodeSelectionState::NoRoute, Some(_)) => {}
                (NodeSelectionState::FindingNode1, Some(n)) => {
                    *state = NodeSelectionState::FindingNode2(n.0);
                }
                (NodeSelectionState::FindingNode2(n0), Some(n)) => {
                    let source = *n0;
                    let target = n.0;
                    let graph = graph.0.clone();
                    let distance_map = road_map.clone();
                    let pool = AsyncComputeTaskPool::get();
                    let task = pool.spawn(async move {
                        find_route(
                            source,
                            target,
                            &graph,
                            |e| distance_map.road_length(*e),
                            |_, _| 0.0,
                        )
                    });
                    *state = NodeSelectionState::FindingRoute(task);
                }
                _ => {}
            }
        }

        let (n1, n2) = match state.as_ref() {
            NodeSelectionState::NoRoute => ("No route".to_string(), "".to_string()),
            NodeSelectionState::FindingNode1 => ("Finding node 1".to_string(), "".to_string()),
            NodeSelectionState::FindingNode2(n1) => (format!("Node: {:?}", n1), "".to_string()),
            NodeSelectionState::FindingRoute(_) => ("Searching route".to_string(), "".to_string()),
            NodeSelectionState::FoundRoute(_) => ("Found route".to_string(), "".to_string()),
        };

        ui.label(n1);
        ui.label(n2);
    });
}

fn waiting_for_task(mut route_state: ResMut<NodeSelectionState>) {
    if let NodeSelectionState::FindingRoute(task) = route_state.as_mut() {
        if let Some(task) = future::block_on(future::poll_once(task)) {
            let state = match task {
                Ok(a) => NodeSelectionState::FoundRoute(a),
                Err(_) => NodeSelectionState::NoRoute,
            };
            *route_state = state
        };
    }
}

fn route_draw(
    route_state: Res<NodeSelectionState>,
    mut query: Query<&mut WorldEntity>,
    network: Res<DirectedNetworkGraphContainer>,
) {
    if let NodeSelectionState::FoundRoute(route) = route_state.as_ref() {
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
