use crate::{
    camera::MainCamera,
    nwb::{self, NwbGraph},
    ui::{DirectedNetworkGraphContainer, PreProcess},
};
use bevy::prelude::*;
use bevy_polyline::prelude::{Polyline, PolylineBundle, PolylineMaterial};
use bevy_shapefile::{RoadId, RoadMap, RoadSection, AABB};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

pub struct WorldPlugin {
    pub config: WorldConfig,
}

#[derive(Resource)]
pub struct LoadedMaterials {
    normal_material: Handle<PolylineMaterial>,
    selected_material: Handle<PolylineMaterial>,

    outgoing_material: Handle<PolylineMaterial>,
    incoming_material: Handle<PolylineMaterial>,
    route_material: Handle<PolylineMaterial>,
}

#[derive(Debug, Clone, Resource)]
pub struct WorldConfig {
    pub database_path: String,
    pub road_map_path: String,
    pub shapefile_path: String,
    pub directed_graph_path: String,

    pub selected_colour: Color,
    pub normal_colour: Color,
}

#[derive(Debug, Clone, Component)]
pub struct WorldEntity {
    pub id: RoadId,
    pub selected: WorldEntitySelectionType,
}

#[derive(Debug, Clone)]
pub enum WorldEntitySelectionType {
    NotSelected,
    BaseSelected,
    BiDirection,
    Outgoing,
    Incoming,
    Route,
}

#[derive(Debug, Default, Resource)]
pub struct WorldTracker {
    pub map: HashMap<RoadId, Entity>,
}

impl WorldTracker {
    pub fn track(&mut self, id: RoadId, entity: Entity) {
        self.map.insert(id, entity);
    }
    pub fn remove(&mut self, id: RoadId) {
        self.map.remove(&id);
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(WorldTracker::default())
            .insert_resource(self.config.clone())
            .add_startup_system(init_materials)
            .add_startup_system(init_road_map)
            .add_system(mark_on_changed_preprocess)
            .add_system(colour_system) // Used for drawing the layers
            // .add_system(test_algorithm)
            // .add_system(help)
            .add_system(visible_entities);
    }
}

fn init_materials(
    config: Res<WorldConfig>,
    mut commands: Commands,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
) {
    let normal_material = polyline_materials.add(PolylineMaterial {
        width: 0.2,
        color: config.normal_colour,
        perspective: true,
        ..Default::default()
    });
    let selected_material = polyline_materials.add(PolylineMaterial {
        width: 3.0,
        color: config.selected_colour,
        ..Default::default()
    });

    let incoming_material = polyline_materials.add(PolylineMaterial {
        width: 3.0,
        color: Color::RED,

        ..Default::default()
    });

    let outgoing_material = polyline_materials.add(PolylineMaterial {
        width: 3.0,
        color: Color::YELLOW,
        ..Default::default()
    });

    let route_material = polyline_materials.add(PolylineMaterial {
        width: 2.0,
        color: Color::ALICE_BLUE,
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

fn init_road_map(config: Res<WorldConfig>, mut commands: Commands) {
    let road_map = load_road_map(&config);

    let network = load_graph(config, &road_map);

    println!("Inserted resources");

    println!("Status:");
    println!("Nodes: {}", network.node_count());
    println!("Edges: {}", network.edge_count());

    commands.insert_resource(road_map);
    commands.insert_resource(DirectedNetworkGraphContainer(network));

    // commands.insert_resource(next_level_edges);
}

fn load_road_map(config: &Res<WorldConfig>) -> RoadMap {
    let road_map_path = Path::new(&config.road_map_path);
    let road_map = if let Ok(road_map) = crate::read_file(road_map_path) {
        road_map
    } else {
        println!("File {road_map_path:?} not found, creating...");
        let road_map = bevy_shapefile::from_shapefile(&config.shapefile_path)
            .expect("Could not read shapefile");

        crate::write_file(&road_map, road_map_path).expect("Could not write road_map");

        road_map
    };
    road_map
}

fn load_graph(config: Res<WorldConfig>, road_map: &RoadMap) -> NwbGraph {
    let network_path = Path::new(&config.directed_graph_path);
    let network: NwbGraph = if let Ok(network) = crate::read_file(network_path) {
        network
    } else {
        println!("File {network_path:?} not found, creating...");
        let network = nwb::preprocess_roadmap(road_map, &config.database_path);
        crate::write_file(&network, network_path).expect("Could not write network");
        network
    };
    network
}

fn mark_on_changed_preprocess(
    mut tracker: ResMut<WorldTracker>,
    preprocess: Option<Res<PreProcess>>,
    mut q_camera: Query<(&Camera, &GlobalTransform, &mut Transform), (With<MainCamera>,)>,
) {
    if let Some(preprocess) = preprocess {
        if preprocess.is_added() && q_camera.get_single_mut().is_ok() {
            tracker.map.clear();
        }
    }
}

fn visible_entities(
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
    if let Ok((camera, transform)) = q_camera.get_single() {
        let min = convert(Vec2::new(-1.0, -1.0), transform, camera);
        let max = convert(Vec2::new(1.0, 1.0), transform, camera);

        let visible = road_map
            .road_spatial
            .locate_in_envelope_intersecting(&AABB::from_corners([min.x, min.y], [max.x, max.y]))
            .map(|x| x.id)
            .collect::<HashSet<_>>();
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
            let section = road_map.roads.get(&id).unwrap();
            let entity = spawn_figure(&mut commands, id, section, &mut polylines, &materials);

            tracker.track(id, entity);
        }
    }
}

fn colour_system(
    loaded_materials: Res<LoadedMaterials>,
    mut query: Query<(&mut WorldEntity, &mut Handle<PolylineMaterial>)>,
) {
    query.par_for_each_mut(32, |(mut we, mut mode)| {
        let material = match we.selected {
            WorldEntitySelectionType::NotSelected => loaded_materials.normal_material.clone_weak(),
            WorldEntitySelectionType::BaseSelected => {
                loaded_materials.selected_material.clone_weak()
            }
            WorldEntitySelectionType::BiDirection => {
                loaded_materials.selected_material.clone_weak()
            }
            WorldEntitySelectionType::Outgoing => loaded_materials.outgoing_material.clone_weak(),
            WorldEntitySelectionType::Incoming => loaded_materials.incoming_material.clone_weak(),
            WorldEntitySelectionType::Route => loaded_materials.route_material.clone_weak(),
        };
        *mode = material;
        we.selected = WorldEntitySelectionType::NotSelected;
    });
}

pub fn convert(pos: Vec2, transform: &GlobalTransform, camera: &Camera) -> Vec2 {
    camera
        .ndc_to_world(transform, pos.extend(0.0))
        .unwrap_or(Vec3::ZERO)
        .truncate()
}

fn spawn_figure(
    commands: &mut Commands,
    id: RoadId,
    section: &RoadSection,
    polylines: &mut Assets<Polyline>,
    materials: &LoadedMaterials,
) -> Entity {
    let polyline = polylines.add(Polyline {
        vertices: section
            .points
            .iter()
            .map(|c| Vec3::new(c.x, c.y, 0.0))
            .collect(),
    });
    commands
        .spawn(PolylineBundle {
            polyline,
            material: materials.normal_material.clone_weak(),
            ..Default::default()
        })
        .insert(WorldEntity {
            id,
            selected: WorldEntitySelectionType::NotSelected,
        })
        .id()
}
