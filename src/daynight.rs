use bevy::prelude::*;
use crate::camera::GameCamera;
use crate::hud::not_paused;

pub struct DayNightPlugin;

impl Plugin for DayNightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DayNightCycle::default())
            .add_systems(Startup, spawn_overlay)
            .add_systems(Update, (update_day_night, apply_ambient_light).run_if(not_paused));
    }
}

#[derive(Resource)]
pub struct DayNightCycle {
    pub time_of_day: f32,
    pub day_count: u32,
    pub day_length_seconds: f32,
    pub paused: bool,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: 0.35,
            day_count: 1,
            day_length_seconds: 600.0,
            paused: false,
        }
    }
}

impl DayNightCycle {
    pub fn phase(&self) -> DayPhase {
        match self.time_of_day {
            t if t < 0.2 => DayPhase::Night,
            t if t < 0.3 => DayPhase::Sunrise,
            t if t < 0.7 => DayPhase::Day,
            t if t < 0.8 => DayPhase::Sunset,
            _ => DayPhase::Night,
        }
    }

    pub fn phase_name(&self) -> &str {
        match self.phase() {
            DayPhase::Night => "Night",
            DayPhase::Sunrise => "Sunrise",
            DayPhase::Day => "Day",
            DayPhase::Sunset => "Sunset",
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DayPhase {
    Night,
    Sunrise,
    Day,
    Sunset,
}

#[derive(Component)]
pub struct DayNightOverlay;

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        DayNightOverlay,
        Sprite {
            color: Color::srgba(0.05, 0.05, 0.2, 0.0),
            custom_size: Some(Vec2::new(4096.0, 4096.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 50.0),
    ));
}

fn update_day_night(
    mut cycle: ResMut<DayNightCycle>,
    time: Res<Time>,
) {
    if cycle.paused {
        return;
    }
    cycle.time_of_day += time.delta_secs() / cycle.day_length_seconds;
    if cycle.time_of_day >= 1.0 {
        cycle.time_of_day -= 1.0;
        cycle.day_count += 1;
    }
}

fn apply_ambient_light(
    cycle: Res<DayNightCycle>,
    mut overlay_query: Query<(&mut Sprite, &mut Transform), (With<DayNightOverlay>, Without<GameCamera>)>,
    camera_query: Query<&Transform, With<GameCamera>>,
) {
    let Ok((mut sprite, mut overlay_tf)) = overlay_query.get_single_mut() else { return };
    let Ok(cam_tf) = camera_query.get_single() else { return };

    // Follow camera so overlay always covers the screen
    overlay_tf.translation.x = cam_tf.translation.x;
    overlay_tf.translation.y = cam_tf.translation.y;

    let darkness = match cycle.phase() {
        DayPhase::Night => 0.55,
        DayPhase::Sunrise => {
            let f = (cycle.time_of_day - 0.2) / 0.1;
            0.55 * (1.0 - f)
        }
        DayPhase::Day => 0.0,
        DayPhase::Sunset => {
            let f = (cycle.time_of_day - 0.7) / 0.1;
            0.55 * f
        }
    };

    sprite.color = Color::srgba(0.02, 0.02, 0.12, darkness);
}
