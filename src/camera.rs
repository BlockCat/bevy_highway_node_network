use bevy::prelude::*;

use crate::geo_coords::{RijkDriehoekCoordinate, WGS84};

#[derive(Component)]
pub struct MainCamera;

pub struct CameraPlugin {
    pub config: CameraConfig,
}

#[derive(Debug, Clone)]
pub struct CameraConfig {
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub zoom_factor: f32,
    pub speed: f32,
    pub left: KeyCode,
    pub right: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.config.clone())
            .add_startup_system(load_camera)
            .add_system(camera_system_zoom)
            .add_system(camera_system_move);
    }
}

fn load_camera(mut commands: Commands) {
    let utrecht = WGS84 {
        latitude: 52.093_597,
        longitude: 5.1134345,
    };
    let utrecht = RijkDriehoekCoordinate::from(utrecht);
    let mut utrecht = Vec3::from(utrecht);
    utrecht.z = 10.0;

    commands
        .spawn_bundle(OrthographicCameraBundle {
            transform: Transform::from_translation(utrecht),
            ..OrthographicCameraBundle::new_2d()
        })
        .insert(MainCamera);
}

fn camera_system_zoom(
    config: Res<CameraConfig>,
    keys: Res<Input<KeyCode>>,
    mut q_camera: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let mut zoomed = false;
    let mut factor = 1.0;

    if keys.pressed(config.zoom_in) {
        factor *= config.zoom_factor;
        zoomed = true;
    }
    if keys.pressed(config.zoom_out) {
        factor /= config.zoom_factor;
        zoomed = true;
    }

    if zoomed {
        let mut projection = q_camera.single_mut();
        projection.scale *= factor;
    }
}

fn camera_system_move(
    config: Res<CameraConfig>,
    keys: Res<Input<KeyCode>>,
    mut q_camera: Query<(&OrthographicProjection, &mut Transform), With<MainCamera>>,
) {
    let mut translation = Vec3::new(0.0, 0.0, 0.0);
    let mut moved = false;

    if keys.pressed(config.left) {
        translation.x -= config.speed;
        moved = true;
    }
    if keys.pressed(config.right) {
        translation.x += config.speed;
        moved = true;
    }
    if keys.pressed(config.up) {
        translation.y += config.speed;
        moved = true;
    }
    if keys.pressed(config.down) {
        translation.y -= config.speed;
        moved = true;
    }

    if moved {
        let (projection, mut transform) = q_camera.single_mut();
        transform.translation += translation * projection.scale;

        println!("Moved: {:?}", transform.translation);
    }
}
