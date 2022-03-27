use crate::camera::MainCamera;
use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes};
use bevy_shapefile::{RoadMap, RoadSection, AABB};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

pub struct WorldPlugin {
    pub config: WorldConfig,
}

#[derive(Debug, Clone)]
pub struct WorldConfig {
    pub compiled_path: String,
    pub data_path: String,

    pub selected_colour: Color,
    pub normal_colour: Color,
}

#[derive(Debug, Clone, Default, Component)]
pub struct WorldEntity {
    pub selected: bool,
}

#[derive(Debug, Default)]
pub struct WorldTracker {
    map: HashMap<usize, Entity>,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(WorldTracker::default())
            .insert_resource(self.config.clone())
            .add_startup_system(init_road_map)
            .add_system(visible_entities);
    }
}

fn init_road_map(config: Res<WorldConfig>, mut commands: Commands) {
    let path = Path::new(&config.compiled_path);

    let road_map = if path.exists() {
        println!("compiled.data exists, reading");
        let road_map = bevy_shapefile::RoadMap::read(path);
        println!("Finished reading");
        road_map
    } else {
        println!("compiled.data doesn't exist, creating");
        let road_map =
            bevy_shapefile::load_file(&config.data_path).expect("Could not read shapefile");

        println!("Finished creating, writing");

        road_map.write(path);

        println!("Finished writing");

        road_map
    };

    commands.insert_resource(road_map);

    println!("Inserted resources");
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
            .map
            .locate_in_envelope_intersecting(&AABB::from_corners([min.x, min.y], [max.x, max.y]))
            .map(|x| x.id)
            .collect::<HashSet<usize>>();

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
            let entity = spawn_figure(&mut commands, section, config.selected_colour.clone());

            tracker.map.insert(*id, entity);
        }
    }
}

fn convert(pos: Vec2, transform: &GlobalTransform, camera: &Camera) -> Vec2 {
    (transform.compute_matrix() * camera.projection_matrix.inverse())
        .project_point3(pos.extend(-1.0))
        .truncate()
}

fn spawn_figure(commands: &mut Commands, section: &RoadSection, color: Color) -> Entity {
    let shape = shapes::Polygon {
        closed: false,
        points: section.points.clone(),
    };
    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Stroke(StrokeMode::new(color, 3.0)),
            Transform::default(),
        ))
        .insert(WorldEntity::default())
        .id()
}
