use super::DirectedNetworkGraphContainer;
use super::PointClickedEvent;
use crate::world::WorldEntity;
use crate::world::WorldEntitySelectionType;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::tasks::Task;
use bevy_egui::{egui, EguiContexts};
use bevy_shapefile::RoadMap;
use futures_lite::future;
use graph::{DirectedNetworkGraph, EdgeId, NetworkData, NodeId, F32};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

pub struct RouteUIPlugin;

impl Plugin for RouteUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NodeSelectionState::default())
            .add_systems(Update, gui_system)
            .add_systems(Update, waiting_for_task)
            .add_systems(Update, route_draw);
    }
}

#[derive(Debug, Default, Resource)]
pub enum NodeSelectionState {
    #[default]
    NoRoute,
    FindingNode1,
    FindingNode2(NodeId),
    FindingRoute(Task<Result<Vec<EdgeId>, String>>),
    FoundRoute(Vec<EdgeId>),
}
pub fn gui_system(
    graph: Res<DirectedNetworkGraphContainer>,
    road_map: Res<RoadMap>,
    mut egui_context: EguiContexts,
    mut state: ResMut<NodeSelectionState>,
    mut event_reader: EventReader<PointClickedEvent>,
) {
    egui::Window::new("Routing").show(egui_context.ctx_mut(), |ui| {
        ui.label("Routing");

        if ui.button("Start route").clicked() {
            *state = NodeSelectionState::FindingNode1;
            event_reader.clear();
        } else {
            match (state.as_ref(), event_reader.read().next()) {
                (NodeSelectionState::NoRoute, Some(_)) => {}
                (NodeSelectionState::FindingNode1, Some(n)) => {
                    *state = NodeSelectionState::FindingNode2(n.0);
                }
                (NodeSelectionState::FindingNode2(n0), Some(n)) => {
                    let source = n0.clone();
                    let target = n.0;
                    let graph = graph.0.clone();

                    let pool = AsyncComputeTaskPool::get();
                    let task = pool.spawn(async move { find_route(source, target, &graph) });
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
            NodeSelectionState::FoundRoute(e) => (
                "Found route".to_string(),
                format!(
                    "L: {}",
                    e.iter()
                        .map(|eid| road_map.road_length(*graph.edge_data(*eid)))
                        .sum::<f32>()
                ),
            ),
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
        let l = route
            .iter()
            .map(|e| network.edge_data(*e))
            .collect::<HashSet<_>>();

        query.iter_mut().for_each(|mut a| {
            if l.contains(&a.id) {
                a.selected = WorldEntitySelectionType::Route;
            }
        });
    }
}

fn find_route<D>(
    source: NodeId,
    target: NodeId,
    network: &DirectedNetworkGraph<D>,
) -> Result<Vec<EdgeId>, String>
where
    D: NetworkData,
{
    let mut map = HashMap::new();
    let mut evaluated = 0usize;
    // let mut test = Vec::new();

    let mut heap = BinaryHeap::new();
    explore_node(network, source, &mut heap, 0f32);

    while let Some((_, F32(old_distance), current, parent)) = heap.pop() {
        evaluated += 1;

        if map.contains_key(&current) {
            continue;
        }
        map.insert(current, parent);

        // test.push(parent);
        // println!("GRR: {} > {} of {}", x, old_distance, spare_distance);
        if current == target {
            println!("Evaluated: {}", evaluated);
            let mut path = Vec::new();
            let mut node = current;

            while let Some((parent, edge)) = map.remove(&node) {
                path.push((node, edge));
                node = parent;
            }

            path.reverse();
            return Ok(path.into_iter().map(|(_, edge)| edge).collect());
        }

        explore_node(network, current, &mut heap, old_distance);
    }
    println!("Not found but Evaluated: {}", evaluated);

    Err(String::from("No path found"))
}

fn explore_node<D: NetworkData>(
    network: &DirectedNetworkGraph<D>,
    source: NodeId,
    heap: &mut BinaryHeap<(Reverse<F32>, F32, NodeId, (NodeId, EdgeId))>,
    old_distance: f32,
) {
    for (id, edge) in network.out_edges(source) {
        let target = edge.target();
        let distance = old_distance + edge.weight();

        heap.push((Reverse(F32(distance)), F32(distance), target, (source, id)));
    }
}
