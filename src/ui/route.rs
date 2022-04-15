use crate::{nwb::NWBNetworkData, world::WorldEntity};
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContext;
use network::{iterators::F32, DirectedNetworkGraph, EdgeId, NetworkData, NodeId};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

pub struct RouteUIPlugin;

impl Plugin for RouteUIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RouteState::default())
            .add_system(gui_system)
            .add_system(route_draw);
    }
}

#[derive(Debug, Default)]
pub struct RouteState {
    find_nodes: bool,
    node_1: Option<NodeId>,
    node_2: Option<NodeId>,
    edges: Option<Vec<(NodeId, EdgeId)>>,
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
    network: Res<DirectedNetworkGraph<NWBNetworkData>>,
) {
    if let Some(loaded) = &route_state.edges {
        let l = loaded
            .iter()
            .map(|(_, e)| *network.edge_data(*e) as u32)
            .collect::<HashSet<_>>();

        query.for_each_mut(|mut a| {
            if l.contains(&a.id) {
                a.selected = Some(Color::ALICE_BLUE);
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
    for (id, initial_descendant) in network.out_edges(source) {
        let target = initial_descendant.target();
        let distance = initial_descendant.distance();

        heap.push((
            Reverse(F32(distance + spare(source, target))),
            F32(distance),
            target,
            (source, id),
        ));
    }

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

        for (id, edge) in network.out_edges(current) {
            let target = edge.target();
            let distance = edge.distance() + old_distance;

            heap.push((
                Reverse(F32(distance + spare_distance)),
                F32(distance),
                target,
                (current, id),
            ));
        }
    }
    println!("Not found but Evaluated: {}", evaluated);

    return Err(String::from("No path found"));
}
