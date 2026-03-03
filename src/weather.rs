use bevy::prelude::*;
use rand::Rng;
use crate::daynight::DayNightCycle;
use crate::hud::not_paused;
use crate::season::{Season, SeasonCycle};
use crate::camera::GameCamera;

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WeatherSystem::default())
            .add_systems(Update, (
                advance_weather,
                spawn_weather_particles,
                move_weather_particles,
                despawn_weather_particles,
            ).run_if(not_paused));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Storm,
}

impl Weather {
    pub fn name(&self) -> &str {
        match self {
            Weather::Clear => "Clear",
            Weather::Rain => "Rain",
            Weather::Snow => "Snow",
            Weather::Storm => "Storm",
        }
    }

    /// Returns true if this weather type uses visible particles.
    pub fn has_particles(&self) -> bool {
        !matches!(self, Weather::Clear)
    }

    /// Particle color for this weather type.
    pub fn particle_color(&self) -> Color {
        match self {
            Weather::Clear => Color::srgba(0.0, 0.0, 0.0, 0.0),
            Weather::Rain => Color::srgba(0.4, 0.6, 0.9, 0.6),
            Weather::Snow => Color::srgba(0.9, 0.95, 1.0, 0.8),
            Weather::Storm => Color::srgba(0.3, 0.4, 0.7, 0.7),
        }
    }

    /// Particle size in pixels.
    pub fn particle_size(&self) -> Vec2 {
        match self {
            Weather::Clear => Vec2::ZERO,
            Weather::Rain => Vec2::new(1.0, 5.0),
            Weather::Snow => Vec2::new(3.0, 3.0),
            Weather::Storm => Vec2::new(1.5, 8.0),
        }
    }

    /// Particle fall velocity (pixels/sec), y component is always negative (falling).
    pub fn particle_velocity(&self) -> Vec2 {
        match self {
            Weather::Clear => Vec2::ZERO,
            Weather::Rain => Vec2::new(-10.0, -280.0),
            Weather::Snow => Vec2::new(-20.0, -60.0),
            Weather::Storm => Vec2::new(-80.0, -400.0),
        }
    }

    /// Max simultaneous particles on screen.
    pub fn max_particles(&self) -> usize {
        match self {
            Weather::Clear => 0,
            Weather::Rain => 120,
            Weather::Snow => 80,
            Weather::Storm => 200,
        }
    }

    /// Weighted list of possible weathers for each season.
    /// Returns (weather, weight) pairs.
    fn weighted_for_season(season: Season) -> &'static [(Weather, u32)] {
        match season {
            Season::Spring => &[
                (Weather::Clear, 50),
                (Weather::Rain, 35),
                (Weather::Snow, 5),
                (Weather::Storm, 10),
            ],
            Season::Summer => &[
                (Weather::Clear, 65),
                (Weather::Rain, 20),
                (Weather::Snow, 0),
                (Weather::Storm, 15),
            ],
            Season::Autumn => &[
                (Weather::Clear, 40),
                (Weather::Rain, 40),
                (Weather::Snow, 10),
                (Weather::Storm, 10),
            ],
            Season::Winter => &[
                (Weather::Clear, 30),
                (Weather::Rain, 10),
                (Weather::Snow, 50),
                (Weather::Storm, 10),
            ],
        }
    }

    pub fn random_for_season(season: Season, rng: &mut impl Rng) -> Self {
        let weighted = Self::weighted_for_season(season);
        let total: u32 = weighted.iter().map(|(_, w)| w).sum();
        if total == 0 {
            return Weather::Clear;
        }
        let mut roll = rng.gen_range(0..total);
        for (weather, weight) in weighted {
            if roll < *weight {
                return *weather;
            }
            roll -= weight;
        }
        Weather::Clear
    }
}

#[derive(Resource)]
pub struct WeatherSystem {
    pub current: Weather,
    /// Countdown in real seconds until next weather roll.
    change_timer: f32,
    change_interval: f32,
    /// Tracks last day so we re-evaluate weather on new days.
    last_day: u32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self {
            current: Weather::Clear,
            change_timer: 0.0,
            change_interval: 120.0, // roll every 2 real-time minutes
            last_day: 1,
        }
    }
}

/// Tag component for weather particle sprites.
#[derive(Component)]
pub struct WeatherParticle {
    pub velocity: Vec2,
    /// Lifetime remaining in seconds.
    pub lifetime: f32,
}

// ── Systems ─────────────────────────────────────────────────────────────────

fn advance_weather(
    mut weather: ResMut<WeatherSystem>,
    season: Res<SeasonCycle>,
    cycle: Res<DayNightCycle>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    // Force a new roll at the start of each in-game day.
    let day_changed = cycle.day_count != weather.last_day;
    if day_changed {
        weather.last_day = cycle.day_count;
        weather.current = Weather::random_for_season(season.current, &mut rng);
        weather.change_timer = weather.change_interval;
        return;
    }

    weather.change_timer -= time.delta_secs();
    if weather.change_timer <= 0.0 {
        weather.current = Weather::random_for_season(season.current, &mut rng);
        weather.change_timer = weather.change_interval;
    }
}

fn spawn_weather_particles(
    mut commands: Commands,
    weather: Res<WeatherSystem>,
    particle_query: Query<(), With<WeatherParticle>>,
    camera_query: Query<&Transform, With<GameCamera>>,
) {
    if !weather.current.has_particles() {
        return;
    }

    let Ok(cam_tf) = camera_query.get_single() else { return };

    let existing = particle_query.iter().count();
    let max = weather.current.max_particles();
    if existing >= max {
        return;
    }

    // Spawn a small batch each frame rather than all at once.
    let to_spawn = (max - existing).min(4);
    let mut rng = rand::thread_rng();

    let spread_x = 700.0_f32;
    let spawn_y_offset = 380.0_f32;

    for _ in 0..to_spawn {
        let ox = rng.gen_range(-spread_x..spread_x);
        let oy = rng.gen_range(0.0..spawn_y_offset);
        let x = cam_tf.translation.x + ox;
        let y = cam_tf.translation.y + oy;

        // Give each particle a slightly randomised lifetime so they don't all die together.
        let base_lifetime = match weather.current {
            Weather::Rain => 2.5,
            Weather::Snow => 6.0,
            Weather::Storm => 1.8,
            Weather::Clear => 1.0,
        };
        let lifetime = base_lifetime + rng.gen_range(-0.5..0.5_f32);

        commands.spawn((
            WeatherParticle {
                velocity: weather.current.particle_velocity(),
                lifetime,
            },
            Sprite {
                color: weather.current.particle_color(),
                custom_size: Some(weather.current.particle_size()),
                ..default()
            },
            Transform::from_xyz(x, y, 60.0),
        ));
    }
}

fn move_weather_particles(
    mut particle_query: Query<(&mut WeatherParticle, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut particle, mut tf) in particle_query.iter_mut() {
        tf.translation.x += particle.velocity.x * dt;
        tf.translation.y += particle.velocity.y * dt;
        particle.lifetime -= dt;
    }
}

fn despawn_weather_particles(
    mut commands: Commands,
    weather: Res<WeatherSystem>,
    particle_query: Query<(Entity, &WeatherParticle)>,
) {
    for (entity, particle) in particle_query.iter() {
        // Despawn expired particles or particles that belong to a now-Clear sky.
        if particle.lifetime <= 0.0 || !weather.current.has_particles() {
            commands.entity(entity).despawn();
        }
    }
}
