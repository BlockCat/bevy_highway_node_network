use crate::{
    camera::MainCamera,
    nwb,
    ui::{DirectedNetworkGraphContainer, PreProcess},
};
use bevy::{math::bounding::Aabb2d, prelude::*};
use bevy_polyline::prelude::{
    Polyline, PolylineBundle, PolylineHandle, PolylineMaterial, PolylineMaterialHandle,
};
use bevy_shapefile::{RoadId, RoadMap, RoadSection};
use graph::DirectedNetworkGraph;
use std::{
    collections::HashSet,
    path::Path,
};

use super::resources::{
    LoadedMaterials, WorldConfig, WorldEntity, WorldEntitySelectionType, WorldTracker,
};

pub fn init_materials(
    config: Res<WorldConfig>,
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    let normal_material = polyline_materials.add(PolylineMaterial {
        width: 1.0,
        color: config.normal_colour.to_linear(),
        perspective: true,
        ..Default::default()
    });
    let selected_material = polyline_materials.add(PolylineMaterial {
        width: 3.0,
        color: config.selected_colour.to_linear(),
        ..Default::default()
    });

    let incoming_material = polyline_materials.add(PolylineMaterial {
        width: 3.0,
        color: LinearRgba {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        },

        ..Default::default()
    });

    let outgoing_material = polyline_materials.add(PolylineMaterial {
        width: 3.0,
        color: LinearRgba {
            red: 1.0,
            green: 1.0,
            blue: 0.0,
            alpha: 1.0,
        },
        ..Default::default()
    });

    let route_material = polyline_materials.add(PolylineMaterial {
        width: 9.0,
        color: LinearRgba {
            red: 1.0,
            green: 0.0,
            blue: 1.0,
            alpha: 1.0,
        },
        ..Default::default()
    });
    commands.insert_resource(LoadedMaterials {
        normal_material,
        selected_material,
        incoming_material,
        outgoing_material,
        route_material,
    });
}

pub fn init_road_map(config: Res<WorldConfig>, mut commands: Commands) {
    let road_map = load_road_map(&config);

    let network = load_graph(config, &road_map);

    println!("Inserted resources");

    println!("Status:");
    println!("Nodes: {}", network.nodes().len());
    println!("Edges: {}", network.edges().len());

    commands.insert_resource(road_map);
    commands.insert_resource(DirectedNetworkGraphContainer(network));
}

fn load_road_map(config: &Res<WorldConfig>) -> RoadMap {
    if let Ok(road_map) = crate::io::read_file(&config.road_map_path) {
        road_map
    } else {
        println!("File {:?} not found, creating...", config.road_map_path);
        let road_map =
            RoadMap::from_geopackage(&config.geopackage_path).expect("Could not load road map");

        crate::io::write_file(&road_map, &config.road_map_path).expect("Could not write road_map");

        road_map
    }
}

fn load_graph(
    config: Res<WorldConfig>,
    road_map: &RoadMap,
) -> DirectedNetworkGraph<nwb::NWBNetworkData> {
    let network_path = Path::new(&config.directed_graph_path);

    if let Ok(network) = crate::io::read_file(network_path) {
        network
    } else {
        println!("File {:?} not found, creating...", network_path);
        let network = nwb::preprocess_roadmap(road_map, &config.geopackage_path);
        crate::io::write_file(&network, network_path).expect("Could not write network");
        network
    }
}

pub fn mark_on_changed_preprocess(
    mut tracker: ResMut<WorldTracker>,
    preprocess: Option<Res<PreProcess>>,
    mut q_camera: Query<(&Camera, &GlobalTransform, &mut Transform), (With<MainCamera>,)>,
) {
    if let Some(preprocess) = preprocess {
        if preprocess.is_added() && q_camera.single_mut().is_ok() {
            tracker.map.clear();
        }
    }
}

pub fn visible_entities(
    mut commands: Commands,
    materials: Res<LoadedMaterials>,
    road_map: Res<RoadMap>,
    mut tracker: ResMut<WorldTracker>,
    mut polylines: ResMut<Assets<Polyline>>,
    q_camera: Query<
        (&Camera, &GlobalTransform),
        (
            With<MainCamera>,
            Or<(Changed<Transform>, Changed<Projection>)>,
        ),
    >,
) {
    if let Ok((camera, transform)) = q_camera.single() {
        let min = convert(Vec2::new(-1.0, -1.0), transform, camera);
        let max = convert(Vec2::new(1.0, 1.0), transform, camera);

        let visible = road_map.load_roads_in_aabb2d(&Aabb2d { min, max });

        let tracked = tracker.map.keys().cloned().collect::<HashSet<_>>();

        let added = visible.difference(&tracked).cloned().collect::<Vec<_>>();
        let removed = tracked.difference(&visible).cloned().collect::<Vec<_>>();

        println!(
            "Tracked: {}, Added: {}, Removed: {}, Unchanged: {}",
            tracked.len(),
            added.len(),
            removed.len(),
            tracked.len() - removed.len(),
        );

        for id in removed {
            let entity = tracker.map.get(&id).unwrap();
            commands.entity(*entity).despawn();

            tracker.remove(id);
        }

        for id in added {
            let section = road_map.get_section(&id).unwrap();
            let entity = spawn_figure(&mut commands, id, section, &mut polylines, &materials);

            tracker.track(id, entity);
        }
    }
}

pub fn colour_system(
    loaded_materials: Res<LoadedMaterials>,
    mut query: Query<(&mut WorldEntity, &mut PolylineMaterialHandle)>,
) {
    query.par_iter_mut().for_each(|(mut we, mut mode)| {
        let material = match we.selected {
            WorldEntitySelectionType::NotSelected => loaded_materials.normal_material.clone(),
            WorldEntitySelectionType::BaseSelected => loaded_materials.selected_material.clone(),
            WorldEntitySelectionType::BiDirection => loaded_materials.selected_material.clone(),
            WorldEntitySelectionType::Outgoing => loaded_materials.outgoing_material.clone(),
            WorldEntitySelectionType::Incoming => loaded_materials.incoming_material.clone(),
            WorldEntitySelectionType::Route => loaded_materials.route_material.clone(),
        };
        *mode = PolylineMaterialHandle(material);
        we.selected = WorldEntitySelectionType::NotSelected;
    });
}

pub fn convert(pos: Vec2, transform: &GlobalTransform, camera: &Camera) -> Vec2 {
    camera
        .ndc_to_world(transform, pos.extend(0.0))
        .unwrap()
        .truncate()
}

fn spawn_figure(
    commands: &mut Commands,
    id: RoadId,
    section: &RoadSection,
    polylines: &mut Assets<Polyline>,
    materials: &LoadedMaterials,
) -> Entity {
    commands
        .spawn(PolylineBundle {
            polyline: PolylineHandle(
                polylines.add(Polyline {
                    vertices: section
                        .points
                        .iter()
                        .map(|c| Vec3::new(c.x, c.y, 0.0))
                        .collect(),
                }),
            ),
            material: PolylineMaterialHandle(materials.normal_material.clone()),
            ..Default::default()
        })
        .insert(WorldEntity {
            id,
            selected: WorldEntitySelectionType::NotSelected,
        })
        .id()
}
