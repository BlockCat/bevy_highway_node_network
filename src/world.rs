use crate::{
    camera::MainCamera,
    nwb::{self},
    ui::PreProcess,
};
use bevy::{prelude::*, tasks::ComputeTaskPool};
use bevy_prototype_lyon::{prelude::*, shapes};
use bevy_shapefile::{RoadId, RoadMap, RoadSection, AABB};
use network::DirectedNetworkGraph;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

pub struct WorldPlugin {
    pub config: WorldConfig,
}

#[derive(Debug, Clone)]
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
    pub selected: Option<Color>,
}

#[derive(Debug, Default)]
pub struct WorldTracker {
    pub map: HashMap<RoadId, Entity>,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(WorldTracker::default())
            .insert_resource(self.config.clone())
            .add_startup_system(init_road_map)
            .add_system(mark_on_changed_preprocess)
            .add_system(colour_system) // Used for drawing the layers
            // .add_system(test_algorithm)
            // .add_system(help)
            .add_system(visible_entities);
    }
}

fn init_road_map(config: Res<WorldConfig>, mut commands: Commands) {
    let road_map = load_road_map(&config);

    let network = load_graph(config, &road_map);

    println!("Inserted resources");

    println!("Status:");
    println!("Nodes: {}", network.nodes().len());
    println!("Edges: {}", network.edges().len());

    // let out = network
    //     .nodes()
    //     .iter()
    //     .map(|nn| nn.out_len())
    //     .collect::<Vec<_>>();

    // println!(
    //     "out_edges: [avg: {}, min: {}, max: {}",
    //     out.iter().sum::<usize>() as f32 / out.len() as f32,
    //     out.iter().min().unwrap(),
    //     out.iter().max().unwrap()
    // );

    // let next_level_edges = network::calculate_layer(30, &network, 2.0);
    // println!("Collected phase1 edges: {}", next_level_edges.len());

    commands.insert_resource(road_map);
    commands.insert_resource(network);

    // commands.insert_resource(next_level_edges);
}

fn load_road_map(config: &Res<WorldConfig>) -> RoadMap {
    let road_map_path = Path::new(&config.road_map_path);
    let road_map = if let Ok(road_map) = crate::read_file(road_map_path) {
        road_map
    } else {
        println!("File {:?} not found, creating...", road_map_path);
        let road_map = bevy_shapefile::from_shapefile(&config.shapefile_path)
            .expect("Could not read shapefile");

        crate::write_file(&road_map, road_map_path).expect("Could not write road_map");

        road_map
    };
    road_map
}

fn load_graph(
    config: Res<WorldConfig>,
    road_map: &RoadMap,
) -> DirectedNetworkGraph<nwb::NWBNetworkData> {
    let network_path = Path::new(&config.directed_graph_path);
    let network = if let Ok(network) = crate::read_file(network_path) {
        network
    } else {
        println!("File {:?} not found, creating...", network_path);
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
        if preprocess.is_added() {
            if let Ok(_) = q_camera.get_single_mut() {
                tracker.map.clear();
            }
        }
    }
}

fn visible_entities(
    mut commands: Commands,
    config: Res<WorldConfig>,
    road_map: Res<RoadMap>,
    mut tracker: ResMut<WorldTracker>,
    q_camera: Query<
        (&Camera, &GlobalTransform),
        (
            With<MainCamera>,
            Or<(Changed<Transform>, Changed<OrthographicProjection>)>,
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

        let added = visible.difference(&tracked).collect::<Vec<_>>();
        let removed = tracked.difference(&visible).collect::<Vec<_>>();

        println!(
            "Tracked: {}, Added: {}, Removed: {}, Unchanged: {}",
            tracked.len(),
            added.len(),
            removed.len(),
            tracked.len() - removed.len()
        );

        for id in removed {
            let entity = tracker.map.get(id).unwrap();
            commands.entity(*entity).despawn();

            tracker.map.remove(id);
        }

        for id in added {
            let section = road_map.roads.get(id).unwrap();
            let colour = config.normal_colour;
            let entity = spawn_figure(&mut commands, *id, section, colour);
            tracker.map.insert(*id, entity);
        }
    }
}

fn colour_system(
    pool: Res<ComputeTaskPool>,
    config: Res<WorldConfig>,
    mut query: Query<(&mut WorldEntity, &mut DrawMode)>,
) {
    query.par_for_each_mut(&pool, 32, |(mut we, mut mode)| {
        if let Some(colour) = we.selected {
            *mode = DrawMode::Stroke(StrokeMode::new(colour, 4.0));
        } else {
            *mode = DrawMode::Stroke(StrokeMode::new(config.normal_colour, 2.0));
        }
        we.selected = None;
    });
}

pub fn convert(pos: Vec2, transform: &GlobalTransform, camera: &Camera) -> Vec2 {
    (transform.compute_matrix() * camera.projection_matrix.inverse())
        .project_point3(pos.extend(-1.0))
        .truncate()
}

fn spawn_figure(
    commands: &mut Commands,
    id: RoadId,
    section: &RoadSection,
    color: Color,
) -> Entity {
    let shape = shapes::Polygon {
        closed: false,
        points: section.points.clone(),
    };
    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Stroke(StrokeMode::new(color, 2.0)),
            Transform::default(),
        ))
        .insert(WorldEntity { id, selected: None })
        .id()
}
