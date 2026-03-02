use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use crate::player::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_follow, camera_zoom));
    }
}

#[derive(Component)]
pub struct GameCamera {
    pub zoom_level: f32,
}

const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 3.0;
const CAMERA_LERP_SPEED: f32 = 5.0;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        GameCamera { zoom_level: 1.0 },
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
}

fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let Ok(mut cam_tf) = camera_query.get_single_mut() else { return };

    let target = Vec3::new(
        player_tf.translation.x,
        player_tf.translation.y,
        cam_tf.translation.z,
    );
    cam_tf.translation = cam_tf
        .translation
        .lerp(target, CAMERA_LERP_SPEED * time.delta_secs());
}

fn camera_zoom(
    mut camera_query: Query<(&mut OrthographicProjection, &mut GameCamera)>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    let Ok((mut projection, mut camera)) = camera_query.get_single_mut() else { return };

    for event in scroll_events.read() {
        camera.zoom_level -= event.y * 0.1;
        camera.zoom_level = camera.zoom_level.clamp(MIN_ZOOM, MAX_ZOOM);
        projection.scale = camera.zoom_level;
    }
}
