use crate::{
    geo_coords::{RijkDriehoekCoordinate, WGS84},
    nwb::NWBNetworkData,
    world::{WorldEntity, WorldTracker},
};
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContext;
use network::{iterators::F32, DirectedNetworkGraph, EdgeId, NetworkData, NodeId};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

#[derive(Debug, Default)]
pub struct RouteState {
    find_nodes: bool,
    node_1: Option<NodeId>,
    node_2: Option<NodeId>,
    edges: Option<Vec<(NodeId, EdgeId)>>,
}

pub fn gui_system(mut egui_context: ResMut<EguiContext>, state: ResMut<RouteState>) {
    egui::Window::new("Preprocessing").show(egui_context.ctx_mut(), |ui| {
        ui.label("Routing");

        if ui.button("Start route").clicked() {
            state.find_nodes = false;
            state.node_1 = None;
            state.node_2 = None;
        }
    });
}

fn route_draw(
    route_state: Res<RouteState>,
    mut query: Query<&mut WorldEntity>,
    network: Res<DirectedNetworkGraph<NWBNetworkData>>,
) {
    if let Some(loaded) = route_state.edges {
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

fn test_algorithm(
    mut state: ResMut<RouteState>,
    mut tracker: ResMut<WorldTracker>,
    network: Res<DirectedNetworkGraph<NWBNetworkData>>,
) {
    if !state.loaded {
        println!("Start finding path!?");
        state.loaded = true;
        let pos_a = Vec2::from(RijkDriehoekCoordinate::from(WGS84 {
            latitude: 52.093_597,
            longitude: 5.1134345,
        }));

        let pos_b = Vec2::from(RijkDriehoekCoordinate::from(WGS84 {
            latitude: 52.0600892,
            longitude: 4.4874663,
        }));

        let node_a = network
            .data
            .node_junctions
            .iter()
            .enumerate()
            .min_by_key(|(_, (_, d))| F32(d.distance_squared(pos_a)))
            .map(|x| NodeId::from(x.0))
            .unwrap();

        let node_b = network
            .data
            .node_junctions
            .iter()
            .enumerate()
            .min_by_key(|(_, (_, d))| F32(d.distance_squared(pos_b)))
            .map(|x| NodeId::from(x.0))
            .unwrap();

        let distance = {
            let a = network.node_data(node_a).1;
            let b = network.node_data(node_b).1;

            println!("{:?} ----- {:?}", a, b);
            a.distance(b)
        };

        println!("Found node: {:?} -> {:?} [{}]", node_a, node_b, distance);

        if let Ok(some) = find_route(node_a.into(), node_b.into(), &network, |a, b| {
            let a = network.node_data(a).1;
            let b = network.node_data(b).1;
            a.distance(b)
            // 0.0
        }) {
            // println!("Could find path: {:?}", some);

            state.edges = some;

            tracker.map.clear();
        } else {
            println!("Could not find path");
        }
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
    let stop_distance = spare(source, target) * 2.0;
    let mut map = HashMap::new();
    let mut evaluated = 0usize;
    // let mut test = Vec::new();

    let mut heap = BinaryHeap::new();
    for (id, initial_descendant) in network.out_edges(source) {
        println!("init: {:?}, {:?}", id, initial_descendant);
        let target = initial_descendant.target();
        let distance = initial_descendant.distance();

        heap.push((
            Reverse(F32(distance + spare(source, target))),
            F32(distance),
            target,
            (source, id),
        ));
    }

    while let Some((Reverse(F32(x)), F32(old_distance), current, parent)) = heap.pop() {
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

        // for (id, edge) in network.in_edges(child) {
        //     let target = edge.target();
        //     let distance = edge.distance() + old_distance;

        //     heap.push((
        //         F32(distance + spare(child, target)),
        //         F32(distance),
        //         target,
        //         (child, id),
        //     ));
        // }
    }
    println!("Not found but Evaluated: {}", evaluated);

    return Err(String::from("No path found"));
    // Ok(test)
}
