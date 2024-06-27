use std::{collections::HashMap, path::Path};

use crate::{
    nwb::NWBNetworkData,
    world::{WorldEntity, WorldEntitySelectionType},
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_egui::{egui, EguiContexts};
use bevy_shapefile::RoadId;
use futures_lite::future;
use highway::generation::intermediate_network::IntermediateData;
use graph::{DirectedNetworkGraph, EdgeId, NetworkData};

use super::DirectedNetworkGraphContainer;

#[derive(Debug, Default, Resource)]
pub struct LayerState {
    pub preprocess_layers: usize,
    pub neighbourhood_size: usize,
    pub contraction_factor: f32,
    pub base_selected: bool,
    pub layers_selected: Vec<bool>,
    pub processing: bool,
}

#[derive(Component)]
pub struct ComputeTask<T>(Task<T>);

pub fn gui_system(
    mut commands: Commands,
    mut egui_context: EguiContexts,
    mut state: ResMut<LayerState>,
    preprocess: Option<Res<PreProcess>>,
    base_network: Res<DirectedNetworkGraphContainer>,
) {
    egui::Window::new("Preprocessing").show(egui_context.ctx_mut(), |ui| {
        ui.label("Preprocess");
        ui.add(egui::Slider::new(&mut state.preprocess_layers, 1..=20).text("Layers"));
        ui.add(egui::Slider::new(&mut state.neighbourhood_size, 1..=90).text("Neighbourhood size"));

        if state.processing {
            ui.add(egui::Spinner::new());
        } else if ui.button("Start Preprocess").clicked() {
            let network = base_network.clone();
            let layer_count = state.preprocess_layers;
            let neighbourhood_size = state.neighbourhood_size;
            let contraction_factor = state.contraction_factor;

            let task = AsyncComputeTaskPool::get().spawn(async move {
                clicked_preprocess(network, layer_count, neighbourhood_size, contraction_factor)
            });
            state.processing = true;

            commands.spawn(ComputeTask(task));
        }

        if let Some(preprocess) = preprocess {
            ui.add(egui::Checkbox::new(&mut state.base_selected, "Layer: 1"));

            for i in 0..preprocess.layers.len() {
                let rekt = format!("Layer: {}", i + 2);
                ui.add(egui::Checkbox::new(&mut state.layers_selected[i], rekt));
            }
        }
    });
}

fn load_or_calculate<P: AsRef<Path>, F>(
    path: P,
    calculate: F,
) -> DirectedNetworkGraph<IntermediateData>
where
    F: Fn() -> DirectedNetworkGraph<IntermediateData>,
{
    if let Ok(network) = crate::read_file(&path) {
        network
    } else {
        let network = calculate();
        crate::write_file(&network, path).expect("Could not write");

        network
    }
}

pub fn handle_preprocess_task(
    mut commands: Commands,
    mut state: ResMut<LayerState>,
    mut query: Query<(Entity, &mut ComputeTask<PreProcess>)>,
) {
    if let Ok((entity, mut task)) = query.get_single_mut() {
        if let Some(preprocess) = future::block_on(future::poll_once(&mut task.0)) {
            state.processing = false;

            state.base_selected = false;
            state.layers_selected = vec![false; preprocess.layers.len()];

            commands.insert_resource(preprocess);

            commands.entity(entity).despawn();
        }
    }
}

fn clicked_preprocess(
    base: DirectedNetworkGraph<NWBNetworkData>,
    layer_count: usize,
    neighbourhood: usize,
    contraction_factor: f32,
) -> PreProcess {
    println!("Clicked: {}", layer_count);

    let mut layers = Vec::new();

    layers.push(load_or_calculate("data/layer_0.graph", || {
        highway::generation::calculate_layer(neighbourhood, &base, contraction_factor)
    }));

    println!(
        "Base edges: {}, nodes: {}",
        base.edges().len(),
        base.nodes().len()
    );

    for i in 1..layer_count {
        let size = neighbourhood;
        let network = layers.last().unwrap();
        let path = format!("data/layer_{}.graph", i);
        let next_layer = load_or_calculate(path, || {
            highway::generation::calculate_layer(size, network, contraction_factor)
        });

        println!(
            "Layer {} edges: {}/{}, nodes: {}/{}",
            i,
            next_layer.edges().len(),
            network.edges().len() as f32 / next_layer.edges().len() as f32,
            next_layer.nodes().len(),
            network.nodes().len() as f32 / next_layer.nodes().len() as f32
        );

        layers.push(next_layer);
    }

    PreProcess::new(base, layers)
}

pub fn colouring_system(
    ui_state: Res<LayerState>,
    preprocess: Option<Res<PreProcess>>,
    mut query: Query<&mut WorldEntity>,
) {
    if let Some(preprocess) = preprocess {
        if ui_state.base_selected {
            query.par_iter_mut().for_each(|mut we| {
                we.selected = WorldEntitySelectionType::BaseSelected;
            });
        } else {
            query.par_iter_mut().for_each(|mut we| {
                for (i, sel) in ui_state.layers_selected.iter().enumerate() {
                    if *sel && preprocess
                            .road_data_level
                            .get(&we.id)
                            .map(|&x| x > i as u8)
                            .unwrap_or_default() {
                        we.selected = WorldEntitySelectionType::BaseSelected;
                    }
                }
            });
        }
    }
}

#[derive(Resource)]
pub struct PreProcess {
    pub base: DirectedNetworkGraph<NWBNetworkData>,
    pub layers: Vec<DirectedNetworkGraph<IntermediateData>>,
    pub road_data_level: HashMap<RoadId, u8>,
}

impl PreProcess {
    pub fn new(
        base: DirectedNetworkGraph<NWBNetworkData>,
        layers: Vec<DirectedNetworkGraph<IntermediateData>>,
    ) -> Self {
        let mut road_data_level = (0..base.edges().len())
            .map(EdgeId::from)
            .flat_map(|id| Vec::from(base.data.edge_road_id(id)))
            .map(|x| (RoadId::from(x), 0))
            .collect::<HashMap<_, _>>();

        println!("Base line of: {}", road_data_level.len());
        // process_edges(0, &base, &mut road_data_level);

        for (layer_id, layer) in layers.iter().enumerate() {
            process_edges(layer_id as u8 + 1, layer, &mut road_data_level);
        }

        PreProcess {
            base,
            layers,
            road_data_level,
        }
    }
}

fn process_edges<A: NetworkData>(
    layer_id: u8,
    network: &DirectedNetworkGraph<A>,
    road_data: &mut HashMap<RoadId, u8>,
) {
    for id in 0..network.edges().len() {
        let id = EdgeId::from(id);
        match network.data.edge_road_id(id) {
            graph::ShortcutState::Single(a) => {
                road_data.entry(RoadId::from(a)).and_modify(|f| {
                    *f = layer_id;
                });
            }
            graph::ShortcutState::Shortcut(b) => {
                for a in b {
                    road_data.entry(RoadId::from(a)).and_modify(|f| {
                        *f = layer_id;
                    });
                }
            }
        }
    }
}
