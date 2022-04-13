use std::collections::HashMap;

use crate::{nwb::NWBNetworkData, world::WorldEntity};
use bevy::{prelude::*, tasks::ComputeTaskPool};
use bevy_egui::{egui, EguiContext};
use network::{intermediate_network::IntermediateData, DirectedNetworkGraph};

#[derive(Debug, Default)]
pub struct LayerState {
    pub preprocess_layers: usize,
    pub neighbourhood_size: usize,
    pub contraction_factor: f32,
    pub base_selected: bool,
    pub layers_selected: Vec<bool>,
}

pub fn gui_system(
    mut commands: Commands,
    base_network: Res<DirectedNetworkGraph<NWBNetworkData>>,
    preprocess: Option<Res<PreProcess>>,
    mut egui_context: ResMut<EguiContext>,
    mut state: ResMut<LayerState>,
) {
    egui::Window::new("Preprocessing").show(egui_context.ctx_mut(), |ui| {
        ui.label("Preprocess");
        ui.add(egui::Slider::new(&mut state.preprocess_layers, 1..=20).text("Layers"));
        ui.add(egui::Slider::new(&mut state.neighbourhood_size, 1..=90).text("Neighbourhood size"));
        if ui.button("Start Preprocess").clicked() {
            let preprocess = clicked_preprocess(
                base_network.clone(),
                state.preprocess_layers,
                state.neighbourhood_size,
                state.contraction_factor,
            );

            state.base_selected = false;
            state.layers_selected = vec![false; preprocess.layers.len()];

            commands.insert_resource(preprocess);
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

fn clicked_preprocess(
    base: DirectedNetworkGraph<NWBNetworkData>,
    layer_count: usize,
    neighbourhood: usize,
    contraction_factor: f32,
) -> PreProcess {
    println!("Clicked: {}", layer_count);

    let mut layers = Vec::new();

    layers.push(network::calculate_layer(
        neighbourhood,
        &base,
        contraction_factor,
    ));

    for _ in 1..layer_count {
        let size = neighbourhood;
        let network = layers.last().unwrap();
        let next_layer = network::calculate_layer(size, network, contraction_factor);
        layers.push(next_layer);
    }

    PreProcess::new(base, layers)
}

pub fn colouring_system(
    pool: Res<ComputeTaskPool>,
    ui_state: Res<LayerState>,
    preprocess: Option<Res<PreProcess>>,
    mut query: Query<&mut WorldEntity>,
) {
    if let Some(preprocess) = preprocess {
        if ui_state.base_selected {
            query.par_for_each_mut(&pool, 32, |mut we| {
                we.selected = Some(Color::GREEN);
            });
        } else {
            for (i, sel) in ui_state.layers_selected.iter().enumerate() {
                query.par_for_each_mut(&pool, 32, |mut we| {
                    if *sel {
                        if preprocess
                            .road_data_level
                            .get(&we.id as &u32)
                            .map(|&x| x > i as u8)
                            .unwrap_or_default()
                        {
                            we.selected = Some(Color::GREEN);
                        }
                    }
                });
            }
        }
    }
}

pub struct PreProcess {
    pub base: DirectedNetworkGraph<NWBNetworkData>,
    pub layers: Vec<DirectedNetworkGraph<IntermediateData>>,
    pub road_data_level: HashMap<u32, u8>,
}

impl PreProcess {
    pub fn new(
        base: DirectedNetworkGraph<NWBNetworkData>,
        layers: Vec<DirectedNetworkGraph<IntermediateData>>,
    ) -> Self {
        let mut road_data_level = HashMap::new();

        for id in 0..base.edges().len() {
            let edge = base.edge(id.into()).edge_id;
            road_data_level.insert(edge, 0u8);
        }

        for (_, layer) in layers.iter().enumerate() {
            for i in 0..layer.edges().len() {
                match layer.edge_data(i.into()) {
                    network::ShortcutState::Single(a) => {
                        road_data_level.entry(*a).and_modify(|f| {
                            *f = i as u8 + 1;
                        });
                    }
                    network::ShortcutState::Shortcut(b) => {
                        for a in b {
                            road_data_level.entry(*a).and_modify(|f| {
                                *f = i as u8 + 1;
                            });
                        }
                    }
                }
            }
        }

        PreProcess {
            base,
            layers,
            road_data_level,
        }
    }
}
