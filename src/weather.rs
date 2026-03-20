use bevy::prelude::*;
use rand::Rng;
use crate::daynight::DayNightCycle;
use crate::hud::{not_paused, WeatherEffectsTimer};
use crate::season::{Season, SeasonCycle};
use crate::camera::GameCamera;
use crate::player::{Player, Health};
use crate::building::{Building, BuildingType};
use crate::combat::Enemy;
use crate::world::{TILE_SIZE, chunk::{Chunk, CHUNK_SIZE}, tile::TileType};
use crate::hud::FloatingTextRequest;
use crate::camera::CameraEffects;

pub struct WeatherPlugin;

/// Frame counter used to skip weather particle spawning every other frame.
#[derive(Resource, Default)]
struct WeatherSpawnFrame(u32);

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WeatherSystem::default())
            .insert_resource(WeatherSpawnFrame::default())
            .add_systems(Update, (
                advance_weather,
                spawn_weather_particles,
                move_weather_particles,
                despawn_weather_particles,
                weather_gameplay_effects,
                lightning_strike_system,
            ).run_if(not_paused));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Storm,
    Fog,
    Blizzard,
}

impl Weather {
    pub fn name(&self) -> &str {
        match self {
            Weather::Clear => "Clear",
            Weather::Rain => "Rain",
            Weather::Snow => "Snow",
            Weather::Storm => "Storm",
            Weather::Fog => "Fog",
            Weather::Blizzard => "Blizzard",
        }
    }

    /// Returns true if this weather type uses visible particles.
    pub fn has_particles(&self) -> bool {
        !matches!(self, Weather::Clear | Weather::Fog)
    }

    /// Particle color for this weather type.
    pub fn particle_color(&self) -> Color {
        match self {
            Weather::Clear => Color::srgba(0.0, 0.0, 0.0, 0.0),
            Weather::Rain => Color::srgba(0.4, 0.6, 0.9, 0.6),
            Weather::Snow => Color::srgba(0.9, 0.95, 1.0, 0.8),
            Weather::Storm => Color::srgba(0.3, 0.4, 0.7, 0.7),
            Weather::Fog => Color::srgba(0.0, 0.0, 0.0, 0.0),
            Weather::Blizzard => Color::srgba(0.85, 0.9, 1.0, 0.9),
        }
    }

    /// Particle size in pixels.
    pub fn particle_size(&self) -> Vec2 {
        match self {
            Weather::Clear => Vec2::ZERO,
            Weather::Rain => Vec2::new(1.0, 5.0),
            Weather::Snow => Vec2::new(3.0, 3.0),
            Weather::Storm => Vec2::new(1.5, 8.0),
            Weather::Fog => Vec2::ZERO,
            Weather::Blizzard => Vec2::new(4.0, 4.0),
        }
    }

    /// Particle fall velocity (pixels/sec), y component is always negative (falling).
    pub fn particle_velocity(&self) -> Vec2 {
        match self {
            Weather::Clear => Vec2::ZERO,
            Weather::Rain => Vec2::new(-10.0, -280.0),
            Weather::Snow => Vec2::new(-20.0, -60.0),
            Weather::Storm => Vec2::new(-80.0, -400.0),
            Weather::Fog => Vec2::ZERO,
            Weather::Blizzard => Vec2::new(-100.0, -120.0),
        }
    }

    /// Max simultaneous particles on screen (halved for performance).
    pub fn max_particles(&self) -> usize {
        match self {
            Weather::Clear => 0,
            Weather::Rain => 60,
            Weather::Snow => 40,
            Weather::Storm => 100,
            Weather::Fog => 0,
            Weather::Blizzard => 80,
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
                (Weather::Fog, 0),
                (Weather::Blizzard, 0),
            ],
            Season::Summer => &[
                (Weather::Clear, 65),
                (Weather::Rain, 20),
                (Weather::Snow, 0),
                (Weather::Storm, 15),
                (Weather::Fog, 0),
                (Weather::Blizzard, 0),
            ],
            Season::Autumn => &[
                (Weather::Clear, 35),
                (Weather::Rain, 35),
                (Weather::Snow, 5),
                (Weather::Storm, 10),
                (Weather::Fog, 15),
                (Weather::Blizzard, 0),
            ],
            Season::Winter => &[
                (Weather::Clear, 20),
                (Weather::Rain, 5),
                (Weather::Snow, 40),
                (Weather::Storm, 10),
                (Weather::Fog, 5),
                (Weather::Blizzard, 20),
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
    /// The next weather that will occur after the current timer expires.
    pub next_weather: Option<Weather>,
    /// Lightning strike cooldown timer (seconds until next possible strike).
    pub lightning_timer: f32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self {
            current: Weather::Clear,
            change_timer: 0.0,
            change_interval: 120.0, // roll every 2 real-time minutes
            last_day: 1,
            next_weather: None,
            lightning_timer: 10.0, // 10s intro delay before first strike
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

/// Tag component for lightning strike flash entities.
#[derive(Component)]
pub struct LightningStrike {
    pub lifetime: f32,
    pub damage: f32,
    pub radius: f32,
    pub has_dealt_damage: bool,
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
        // Pre-roll next weather for forecast
        weather.next_weather = Some(Weather::random_for_season(season.current, &mut rng));
        return;
    }

    weather.change_timer -= time.delta_secs();
    if weather.change_timer <= 0.0 {
        // Transition to the pre-rolled next weather if available
        if let Some(next) = weather.next_weather.take() {
            weather.current = next;
        } else {
            weather.current = Weather::random_for_season(season.current, &mut rng);
        }
        weather.change_timer = weather.change_interval;
        // Pre-roll next weather for forecast
        weather.next_weather = Some(Weather::random_for_season(season.current, &mut rng));
    }
}

fn spawn_weather_particles(
    mut commands: Commands,
    weather: Res<WeatherSystem>,
    particle_query: Query<(), With<WeatherParticle>>,
    camera_query: Query<&Transform, With<GameCamera>>,
    mut spawn_frame: ResMut<WeatherSpawnFrame>,
) {
    // Only spawn weather particles every other frame
    spawn_frame.0 = spawn_frame.0.wrapping_add(1);
    if spawn_frame.0 % 2 != 0 {
        return;
    }

    if !weather.current.has_particles() {
        return;
    }

    let Ok(cam_tf) = camera_query.get_single() else { return };

    let existing = particle_query.iter().count();
    let max = weather.current.max_particles();
    if existing >= max {
        return;
    }

    // Spawn max 2 per frame (down from 4)
    let to_spawn = (max - existing).min(2);
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
            Weather::Fog => 1.0,
            Weather::Blizzard => 3.0,
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

// ── US-037: Weather gameplay effects ────────────────────────────────────────

impl Weather {
    /// Farm growth multiplier: rain speeds up farms by 25%.
    #[allow(dead_code)]
    pub fn farm_growth_multiplier(&self) -> f32 {
        match self {
            Weather::Rain => 1.25,
            _ => 1.0,
        }
    }

    /// Movement speed multiplier for snow/storm/blizzard.
    #[allow(dead_code)]
    pub fn movement_speed_multiplier(&self) -> f32 {
        match self {
            Weather::Snow => 0.85,
            Weather::Storm => 0.9,
            Weather::Blizzard => 0.7,
            _ => 1.0,
        }
    }

    /// Enemy speed multiplier during storms/blizzards.
    #[allow(dead_code)]
    pub fn enemy_speed_multiplier(&self) -> f32 {
        match self {
            Weather::Storm => 0.8,
            Weather::Blizzard => 0.6,
            _ => 1.0,
        }
    }
}

/// Check if position is near a Roof building (within 32px).
fn is_under_roof(
    pos: Vec3,
    building_query: &Query<(&Transform, &Building), (Without<Player>, Without<Enemy>)>,
) -> bool {
    for (btf, building) in building_query.iter() {
        if matches!(building.building_type, BuildingType::WoodRoof | BuildingType::StoneRoof) {
            let dist = (btf.translation.truncate() - pos.truncate()).length();
            if dist < 32.0 {
                return true;
            }
        }
    }
    false
}

/// Weather damage effects run 4x/sec (0.25s interval) instead of every frame.
fn weather_gameplay_effects(
    weather: Res<WeatherSystem>,
    season: Res<SeasonCycle>,
    time: Res<Time>,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    building_query: Query<(&Transform, &Building), (Without<Player>, Without<Enemy>)>,
    chunk_query: Query<&Chunk>,
    mut effects_timer: ResMut<WeatherEffectsTimer>,
) {
    effects_timer.0 += time.delta_secs();
    if effects_timer.0 < 0.25 {
        return;
    }
    let accumulated_dt = effects_timer.0;
    effects_timer.0 = 0.0;

    let Ok((player_tf, mut health)) = player_query.get_single_mut() else { return };

    if weather.current == Weather::Storm {
        let under_roof = is_under_roof(player_tf.translation, &building_query);
        if !under_roof {
            let damage_per_sec = 1.0 / 15.0;
            health.current = (health.current - damage_per_sec * accumulated_dt).max(0.0);
        }
    }

    // Blizzard: 3 HP/s cold damage when not under a roof
    if weather.current == Weather::Blizzard {
        let under_roof = is_under_roof(player_tf.translation, &building_query);
        if !under_roof {
            let damage_per_sec = 3.0;
            health.current = (health.current - damage_per_sec * accumulated_dt).max(0.0);
        }
    }

    if season.current == Season::Winter {
        let world_tile_x = (player_tf.translation.x / TILE_SIZE).floor() as i32;
        let world_tile_y = (player_tf.translation.y / TILE_SIZE).floor() as i32;
        let chunk_pos = IVec2::new(
            world_tile_x.div_euclid(CHUNK_SIZE as i32),
            world_tile_y.div_euclid(CHUNK_SIZE as i32),
        );
        let local_x = world_tile_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_tile_y.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_x = local_x.min(CHUNK_SIZE - 1);
        let local_y = local_y.min(CHUNK_SIZE - 1);
        for chunk in chunk_query.iter() {
            if chunk.position == chunk_pos {
                let tile = chunk.get_tile(local_x, local_y);
                if matches!(tile, TileType::Water | TileType::DeepWater) {
                    let damage_per_sec = 2.0;
                    health.current = (health.current - damage_per_sec * accumulated_dt).max(0.0);
                }
                break;
            }
        }
    }
}

// ── Lightning strikes during Storm ──────────────────────────────────────────

fn lightning_strike_system(
    mut commands: Commands,
    mut weather: ResMut<WeatherSystem>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut strike_query: Query<(Entity, &mut LightningStrike, &Transform), Without<Player>>,
    mut player_health_query: Query<&mut Health, With<Player>>,
    mut floating_text_events: EventWriter<FloatingTextRequest>,
    mut effects: ResMut<CameraEffects>,
) {
    let dt = time.delta_secs();

    // Only during Storm
    if weather.current != Weather::Storm {
        weather.lightning_timer = 5.0; // reset to intro delay for next storm
        // Still tick existing strikes
        for (entity, mut strike, _tf) in strike_query.iter_mut() {
            strike.lifetime -= dt;
            if strike.lifetime <= 0.0 {
                commands.entity(entity).despawn();
            }
        }
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Tick lightning timer
    weather.lightning_timer -= dt;
    if weather.lightning_timer <= 0.0 {
        let mut rng = rand::thread_rng();
        // Random offset 20-80px from player
        let angle = rng.gen::<f32>() * std::f32::consts::TAU;
        let dist = rng.gen_range(20.0..80.0);
        let strike_pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * dist;

        // Spawn lightning strike entity (white flash sprite)
        commands.spawn((
            LightningStrike {
                lifetime: 0.3,
                damage: 15.0,
                radius: 8.0,
                has_dealt_damage: false,
            },
            Sprite {
                color: Color::srgba(1.0, 1.0, 0.9, 0.9),
                custom_size: Some(Vec2::new(6.0, 40.0)),
                ..default()
            },
            Transform::from_xyz(strike_pos.x, strike_pos.y, 50.0),
        ));

        // Next strike in 30-60 seconds
        weather.lightning_timer = rng.gen_range(30.0..60.0);

        // Screen shake for the strike
        effects.shake.timer = 0.15;
        effects.shake.intensity = 5.0;
    }

    // Tick existing strikes and apply damage
    for (entity, mut strike, tf) in strike_query.iter_mut() {
        strike.lifetime -= dt;

        if !strike.has_dealt_damage {
            strike.has_dealt_damage = true;
            let strike_pos = tf.translation.truncate();
            let dist_to_player = strike_pos.distance(player_pos);
            if dist_to_player <= strike.radius {
                if let Ok(mut health) = player_health_query.get_single_mut() {
                    health.current = (health.current - strike.damage).max(0.0);
                    floating_text_events.send(FloatingTextRequest {
                        text: format!("-{:.0} LIGHTNING!", strike.damage),
                        position: player_pos + Vec2::new(0.0, 14.0),
                        color: Color::srgb(1.0, 1.0, 0.5),
                    });
                }
            }
        }

        if strike.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
