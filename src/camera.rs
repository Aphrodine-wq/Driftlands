use bevy::prelude::*;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseWheel;
use rand::Rng;
use crate::player::Player;

#[derive(Default)]
pub struct ScreenShake {
    pub timer: f32,
    pub intensity: f32,
}

#[derive(Default)]
pub struct HitStop {
    pub timer: f32,
}

#[derive(Resource, Default)]
pub struct CameraEffects {
    pub shake: ScreenShake,
    pub hit_stop: HitStop,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraEffects>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_follow, camera_zoom, tick_hit_stop));
    }
}

#[derive(Component)]
pub struct GameCamera {
    pub zoom_level: f32,
}

const MIN_ZOOM: f32 = 0.8;
const MAX_ZOOM: f32 = 4.0;
const CAMERA_LERP_SPEED: f32 = 8.0;
const DEAD_ZONE: f32 = 8.0;
const TELEPORT_THRESHOLD: f32 = 500.0;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.008, 0.008, 0.024)),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::default(),
        GameCamera { zoom_level: 1.2 },
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
}

fn camera_follow(
    player_query: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    time: Res<Time>,
    mut effects: ResMut<CameraEffects>,
    settings: Res<crate::settings::GameSettings>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let Ok(mut cam_tf) = camera_query.get_single_mut() else { return };

    let delta = Vec2::new(
        player_tf.translation.x - cam_tf.translation.x,
        player_tf.translation.y - cam_tf.translation.y,
    );
    let distance = delta.length();

    if distance > TELEPORT_THRESHOLD {
        // Instant snap for teleports (dungeon enter/exit)
        cam_tf.translation.x = player_tf.translation.x;
        cam_tf.translation.y = player_tf.translation.y;
    } else if distance >= DEAD_ZONE {
        // Smooth lerp toward player
        let dt = time.delta_secs();
        cam_tf.translation.x += delta.x * CAMERA_LERP_SPEED * dt;
        cam_tf.translation.y += delta.y * CAMERA_LERP_SPEED * dt;
    }
    // If distance < DEAD_ZONE, don't move camera (dead zone)

    // Apply screen shake offset AFTER the lerp (respects settings toggle)
    if effects.shake.timer > 0.0 {
        if settings.screen_shake {
            let mut rng = rand::thread_rng();
            let offset_x = rng.gen_range(-effects.shake.intensity..effects.shake.intensity);
            let offset_y = rng.gen_range(-effects.shake.intensity..effects.shake.intensity);
            cam_tf.translation.x += offset_x;
            cam_tf.translation.y += offset_y;
        }
        effects.shake.timer -= time.delta_secs();
        if effects.shake.timer <= 0.0 {
            effects.shake.timer = 0.0;
            effects.shake.intensity = 0.0;
        }
    }
}

fn camera_zoom(
    mut camera_query: Query<(&mut OrthographicProjection, &mut GameCamera)>,
    mut scroll_events: EventReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok((mut projection, mut camera)) = camera_query.get_single_mut() else { return };

    for event in scroll_events.read() {
        camera.zoom_level -= event.y * 0.1;
        camera.zoom_level = camera.zoom_level.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    // Keyboard zoom: +/= zooms in (lower scale), -/_ zooms out (higher scale)
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        camera.zoom_level = (camera.zoom_level - 0.25).max(MIN_ZOOM);
    }
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        camera.zoom_level = (camera.zoom_level + 0.25).min(MAX_ZOOM);
    }

    projection.scale = camera.zoom_level;
}

fn tick_hit_stop(
    time: Res<Time>,
    mut effects: ResMut<CameraEffects>,
) {
    if effects.hit_stop.timer > 0.0 {
        effects.hit_stop.timer -= time.delta_secs();
        if effects.hit_stop.timer < 0.0 {
            effects.hit_stop.timer = 0.0;
        }
    }
}
