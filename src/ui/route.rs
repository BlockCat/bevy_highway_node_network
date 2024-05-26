use super::DirectedNetworkGraphContainer;
use crate::world::WorldEntity;
use crate::world::WorldEntitySelectionType;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use network::{iterators::F32, DirectedNetworkGraph, EdgeId, NetworkData, NodeId};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

pub struct RouteUIPlugin;

impl Plugin for RouteUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RouteState::default())
            .add_systems(Update, gui_system)
            .add_systems(Update, route_draw);
    }
}

#[derive(Debug, Default, Resource)]
pub struct RouteState {
    find_nodes: bool,
    node_1: Option<NodeId>,
    node_2: Option<NodeId>,
    edges: Option<Vec<(NodeId, EdgeId)>>,
}

pub fn gui_system(mut egui_context: EguiContexts, mut state: ResMut<RouteState>) {
    egui::Window::new("Preprocessing").show(egui_context.ctx_mut(), |ui| {
        ui.label("Routing");

        if ui.button("Start route").clicked() {
            state.find_nodes = false;
            state.node_1 = None;
            state.node_2 = None;
        }

        let n1 = state
            .node_1
            .map(|x| format!("Node: {}", *x))
            .unwrap_or("Node: none".into());
        let n2 = state
            .node_1
            .map(|x| format!("Node: {}", *x))
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
    if let Some(loaded) = &route_state.edges {
        let l = loaded
            .iter()
            .map(|(_, e)| *network.edge_data(*e))
            .collect::<HashSet<_>>();

        query.iter_mut().for_each(|mut a| {
            if l.contains(&a.id) {
                a.selected = WorldEntitySelectionType::Route; // Some(Color::ALICE_BLUE);
            }
        });
    }
}

fn find_route<D, F>(
    source: NodeId,
    target: NodeId,
    network: &DirectedNetworkGraph<D>,
    spare: F,
) -> Result<Vec<(NodeId, EdgeId)>, String>
where
    D: NetworkData,
    F: Fn(NodeId, NodeId) -> f32,
{
    let mut map = HashMap::new();
    let mut evaluated = 0usize;
    // let mut test = Vec::new();

    let mut heap = BinaryHeap::new();
    explore_node(network, source, &mut heap, 0f32, spare(source, target));

    while let Some((_, F32(old_distance), current, parent)) = heap.pop() {
        let spare_distance = spare(current, target);
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
            return Ok(path);
        }

        explore_node(network, current, &mut heap, old_distance, spare_distance);
    }
    println!("Not found but Evaluated: {}", evaluated);

    Err(String::from("No path found"))
}

fn explore_node<D: NetworkData>(
    network: &DirectedNetworkGraph<D>,
    source: NodeId,
    heap: &mut BinaryHeap<(Reverse<F32>, F32, NodeId, (NodeId, EdgeId))>,
    old_distance: f32,
    spare_distance: f32,
) {
    for (id, edge) in network.out_edges(source) {
        let target = edge.target();
        let distance = old_distance + edge.distance();

        heap.push((
            Reverse(F32(distance + spare_distance)),
            F32(distance),
            target,
            (source, id),
        ));
    }
}
