use super::DirectedNetworkGraphContainer;
use crate::{
    nwb::NwbGraph,
    world::{WorldEntity, WorldEntitySelectionType},
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_egui::{egui, EguiContexts};
use bevy_shapefile::{JunctionId, RoadId, RoadMap};
use futures_lite::future;
use network::{iterators::Distanceable, HighwayGraph, Shorted};
use petgraph::visit::EdgeRef;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Default, Resource)]
pub struct LayerState {
    pub preprocess_layers: usize,
    pub neighbourhood_size: usize,
    pub contraction_factor: f32,
    pub base_selected: bool,
    pub layers_selected: Vec<bool>,
    pub processing: bool,
}

#[derive(Debug, Clone)]
struct RoadWeight(RoadId, f32);

impl Distanceable for RoadWeight {
    fn distance(&self) -> f32 {
        self.1
    }
}

#[derive(Component)]
pub struct ComputeTask<T>(Task<T>);

pub fn gui_system(
    mut commands: Commands,
    mut egui_context: EguiContexts,
    mut state: ResMut<LayerState>,
    road_map: Res<RoadMap>,
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
            let road_map = road_map.clone();

            let task = AsyncComputeTaskPool::get().spawn(async move {
                clicked_preprocess(
                    network,
                    road_map,
                    layer_count,
                    neighbourhood_size,
                    contraction_factor,
                )
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
) -> HighwayGraph<(JunctionId, Vec2), Shorted>
where
    F: Fn() -> HighwayGraph<(JunctionId, Vec2), Shorted>,
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
    network: NwbGraph,
    road_map: RoadMap,
    layer_count: usize,
    neighbourhood: usize,
    contraction_factor: f32,
) -> PreProcess {
    println!("Clicked: {layer_count}");

    let base =
        HighwayGraph::from(network.map(|_, n| *n, |_, e| RoadWeight(*e, road_map.road_length(*e))));

    let mut layers = Vec::new();
    layers.push(load_or_calculate("data/layer_0.graph", || {
        highway::generation::calculate_layer(neighbourhood, base.clone(), contraction_factor)
    }));

    println!(
        "Base edges: {}, nodes: {}",
        base.edge_count(),
        base.node_count()
    );

    for i in 1..layer_count {
        let size = neighbourhood;
        let network = layers.last().unwrap();
        let path = format!("data/layer_{i}.graph");
        let next_layer = load_or_calculate(path, || {
            highway::generation::calculate_layer(size, network.clone(), contraction_factor)
        });

        println!(
            "Layer {} edges: {}/{}, nodes: {}/{}",
            i,
            next_layer.edge_count(),
            network.edge_count() as f32 / next_layer.edge_count() as f32,
            next_layer.node_count(),
            network.node_count() as f32 / next_layer.node_count() as f32
        );

        layers.push(next_layer);
    }

    PreProcess::new(network, layers)
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
                    if *sel
                        && preprocess
                            .road_data_level
                            .get(&we.id)
                            .map(|&x| x > i as u8)
                            .unwrap_or_default()
                    {
                        we.selected = WorldEntitySelectionType::BaseSelected;
                    }
                }
            });
        }
    }
}

#[derive(Resource)]
pub struct PreProcess {
    pub base: NwbGraph,
    pub layers: Vec<HighwayGraph<(JunctionId, Vec2), Shorted>>,
    pub road_data_level: HashMap<RoadId, u8>,
}

impl PreProcess {
    pub fn new(
        nwb_graph: NwbGraph,
        layers: Vec<HighwayGraph<(JunctionId, Vec2), Shorted>>,
    ) -> Self {
        let mut road_data_level = nwb_graph
            .edge_weights()
            .map(|e| (*e, 0))
            .collect::<HashMap<_, _>>();

        println!("Base line of: {}", road_data_level.len());

        process_edges(&nwb_graph, &mut road_data_level, &layers);

        PreProcess {
            base: nwb_graph,
            layers,
            road_data_level,
        }
    }
}

/// Finds the highest layer of roads
/// Basically, every road stores the highest level where it can be found
fn process_edges<N>(
    nwb_graph: &NwbGraph,
    road_data: &mut HashMap<RoadId, u8>,
    layers: &Vec<HighwayGraph<N, Shorted>>,
) {
    let base = nwb_graph
        .edge_references()
        .map(|e| (e.id(), vec![*e.weight()]))
        .collect::<HashMap<_, _>>();

    let mut layer_translations = vec![base];

    for level in 0..layers.len() {
        // Add a new layer of translations
        layer_translations.push(HashMap::default());
        assert_eq!(layer_translations.len(), level + 2);

        for edge in layers[level].edge_references() {
            let edge_id = edge.id();
            let weight = edge.weight();
            let road_ids = weight
                .skipped_edges
                .iter()
                .flat_map(|s| {
                    // Level is the same as the previous layer translation
                    layer_translations[level][s].clone()
                })
                .collect::<Vec<_>>();

            for road_id in &road_ids {
                road_data.insert(*road_id, level as u8 + 1);
            }
            layer_translations[level + 1]
                .entry(edge_id)
                .or_default()
                .extend(road_ids);
        }
    }
}
