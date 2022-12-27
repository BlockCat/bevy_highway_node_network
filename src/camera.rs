use bevy::prelude::*;

use crate::geo_coords::{RijkDriehoekCoordinate, WGS84};

#[derive(Component)]
pub struct MainCamera;

pub struct CameraPlugin {
    pub config: CameraConfig,
}

#[derive(Debug, Resource, Clone)]
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
    let utrecht = Vec2::from(utrecht);

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(utrecht.extend(10.0))
                .looking_at(utrecht.extend(0.0), Vec3::Y),
            projection: Projection::Orthographic(Default::default()),
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        })
        .insert(MainCamera);
}

fn camera_system_zoom(
    config: Res<CameraConfig>,
    keys: Res<Input<KeyCode>>,
    mut q_camera: Query<&mut Projection, With<MainCamera>>,
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
        match projection.as_mut() {
            Projection::Perspective(_) => todo!(),
            Projection::Orthographic(projection) => {
                projection.scale *= factor;
                println!("Zoomed: {}", projection.scale);
            }
        }
    }
}

fn camera_system_move(
    config: Res<CameraConfig>,
    keys: Res<Input<KeyCode>>,
    mut q_camera: Query<(&Projection, &mut Transform), With<MainCamera>>,
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

        match projection {
            Projection::Orthographic(projection) => {
                transform.translation += translation * projection.scale;
            }
            Projection::Perspective(_) => todo!(),
        }

        println!("Moved: {:?}", transform.translation);
    }
}
