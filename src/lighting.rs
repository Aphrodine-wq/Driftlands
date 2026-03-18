//! Global lighting settings for normal-mapped 2D lighting.
//! Populated from day/night cycle, weather, biome, and optional point lights.

use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use crate::daynight::{DayNightCycle, DayPhase};
use crate::hud::{not_paused, CurrentBiome};
use crate::player::{Player, Health};
use crate::weather::{Weather, WeatherSystem};
use crate::world::generation::Biome;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LightingSettings::default()).add_systems(
            Update,
            (update_lighting_from_world, collect_point_lights).run_if(not_paused),
        );
    }
}

/// Maximum number of point lights sent to the shader.
pub const MAX_POINT_LIGHTS: usize = 4;

#[derive(Resource, Clone)]
pub struct LightingSettings {
    /// Ambient light color (linear RGB, 0–1). Shader multiplies this with base color.
    pub ambient_color: Vec3,
    /// Direction from surface toward sun (2D, normalized). Used for Lambert term.
    pub sun_direction: Vec2,
    /// Sun/moon color (linear RGB).
    pub sun_color: Vec3,
    /// Sun intensity multiplier (0 at night, ~1 at noon).
    pub sun_intensity: f32,
    /// Point lights (campfires, torches, etc.). Only first MAX_POINT_LIGHTS are used.
    pub point_lights: Vec<PointLight>,
}

impl Default for LightingSettings {
    fn default() -> Self {
        Self {
            ambient_color: Vec3::new(0.08, 0.08, 0.12),
            sun_direction: Vec2::new(0.0, 1.0),
            sun_color: Vec3::new(0.95, 0.9, 0.8),
            sun_intensity: 0.7,
            point_lights: Vec::with_capacity(MAX_POINT_LIGHTS),
        }
    }
}

/// GPU-friendly point light for shaders.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct PointLightUniform {
    pub position: Vec2,
    pub radius: f32,
    pub intensity: f32,
    pub _pad: f32,
    pub color: Vec3,
    pub _pad2: f32,
}

/// GPU-friendly lighting uniform for chunk/sprite shaders.
#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct LightingUniform {
    pub ambient_color: Vec3,
    pub _pad0: f32,
    pub sun_direction: Vec2,
    pub sun_intensity: f32,
    pub sun_color: Vec3,
    pub _pad1: f32,
    pub point_lights: [PointLightUniform; MAX_POINT_LIGHTS],
}

impl LightingUniform {
    pub fn from_settings(s: &LightingSettings) -> Self {
        let mut point_lights = [PointLightUniform::default(); MAX_POINT_LIGHTS];
        for (i, pl) in s.point_lights.iter().take(MAX_POINT_LIGHTS).enumerate() {
            point_lights[i] = PointLightUniform {
                position: pl.position,
                radius: pl.radius,
                intensity: pl.intensity,
                _pad: 0.0,
                color: pl.color,
                _pad2: 0.0,
            };
        }
        Self {
            ambient_color: s.ambient_color,
            _pad0: 0.0,
            sun_direction: s.sun_direction,
            sun_intensity: s.sun_intensity,
            sun_color: s.sun_color,
            _pad1: 0.0,
            point_lights,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PointLight {
    pub position: Vec2,
    pub radius: f32,
    pub color: Vec3,
    pub intensity: f32,
}

/// Updates global lighting from day/night, weather, and biome.
/// In dungeons/caves: limited light radius — very low ambient and sun so torches matter.
fn update_lighting_from_world(
    cycle: Res<DayNightCycle>,
    season: Res<crate::season::SeasonCycle>,
    weather: Res<WeatherSystem>,
    current_biome: Res<CurrentBiome>,
    player_query: Query<&Health, With<Player>>,
    dungeon_registry: Res<crate::dungeon::DungeonRegistry>,
    mut lighting: ResMut<LightingSettings>,
) {
    if dungeon_registry.current_dungeon.is_some() {
        lighting.ambient_color = Vec3::new(0.02, 0.02, 0.03);
        lighting.sun_direction = Vec2::new(0.0, 1.0);
        lighting.sun_color = Vec3::new(0.3, 0.3, 0.35);
        lighting.sun_intensity = 0.05;
        return;
    }

    let phase = cycle.phase_with_season(season.current);
    let t = cycle.time_of_day;

    // --- Sun direction: simple 2D arc (east → high → west) ---
    // 0.0 = night, 0.25 = east/sunrise, 0.5 = noon, 0.75 = west/sunset
    let (sun_direction, sun_color, sun_intensity) = sun_params(&phase, t);

    // --- Ambient: darker at night, with biome and weather tint ---
    let (ambient_r, ambient_g, ambient_b) = ambient_params(&phase, t, &weather, &current_biome);

    // Low health: slightly darker ambient and red tint (optional, subtle)
    let health_frac = player_query
        .get_single()
        .map(|h| (h.current / h.max).clamp(0.0, 1.0))
        .unwrap_or(1.0);
    let low = 1.0 - health_frac;
    let health_ambient_darken = 0.05 * low;
    let health_red = 0.04 * low;

    lighting.sun_direction = sun_direction;
    lighting.sun_color = sun_color;
    lighting.sun_intensity = sun_intensity;
    lighting.ambient_color = Vec3::new(
        (ambient_r - health_ambient_darken + health_red).max(0.0).min(1.0),
        (ambient_g - health_ambient_darken).max(0.0).min(1.0),
        (ambient_b - health_ambient_darken).max(0.0).min(1.0),
    );
}

fn sun_params(phase: &DayPhase, t: f32) -> (Vec2, Vec3, f32) {
    match *phase {
        DayPhase::Night => {
            // Moon: weak, blue-ish, direction from opposite of sun arc
            let dir = Vec2::new(-0.6, 0.8).normalize_or_zero();
            let color = Vec3::new(0.4, 0.45, 0.6);
            let intensity = 0.15;
            (dir, color, intensity)
        }
        DayPhase::Sunrise => {
            let f = ((t - 0.2) / 0.1).clamp(0.0, 1.0);
            let dir = Vec2::new(0.7 + 0.3 * (1.0 - f), 0.5 + 0.5 * f).normalize_or_zero();
            let color = Vec3::new(0.95, 0.6, 0.35).lerp(Vec3::new(0.95, 0.9, 0.8), f * 0.7);
            let intensity = 0.3 + 0.5 * f;
            (dir, color, intensity)
        }
        DayPhase::Day => {
            let dir = Vec2::new(0.0, 1.0);
            let color = Vec3::new(0.95, 0.92, 0.88);
            let intensity = 0.85;
            (dir, color, intensity)
        }
        DayPhase::Sunset => {
            let f = ((t - 0.7) / 0.1).clamp(0.0, 1.0);
            let dir = Vec2::new(-0.7 - 0.3 * f, 0.5 + 0.5 * (1.0 - f)).normalize_or_zero();
            let color = Vec3::new(0.95, 0.9, 0.8).lerp(Vec3::new(0.95, 0.5, 0.3), 1.0 - f);
            let intensity = 0.85 - 0.55 * f;
            (dir, color, intensity)
        }
    }
}

fn ambient_params(
    phase: &DayPhase,
    _t: f32,
    weather: &WeatherSystem,
    current_biome: &CurrentBiome,
) -> (f32, f32, f32) {
    let (r, g, b): (f32, f32, f32) = match *phase {
        DayPhase::Night => (0.04, 0.04, 0.08),
        DayPhase::Sunrise | DayPhase::Sunset => (0.12, 0.08, 0.10),
        DayPhase::Day => (0.18, 0.18, 0.22),
    };

    let weather_darken: f32 = match weather.current {
        Weather::Clear => 0.0,
        Weather::Rain | Weather::Snow => 0.03,
        Weather::Storm => 0.06,
        Weather::Fog => 0.04,
        Weather::Blizzard => 0.07,
    };

    let biome = current_biome.biome.unwrap_or(Biome::Forest);
    let (br, bg, bb): (f32, f32, f32) = match biome {
        Biome::Forest => (0.02, 0.03, 0.01),
        Biome::Coastal => (0.01, 0.03, 0.05),
        Biome::Swamp => (0.02, 0.02, 0.01),
        Biome::Desert => (0.04, 0.03, 0.01),
        Biome::Tundra => (0.02, 0.03, 0.05),
        Biome::Volcanic => (0.04, 0.01, 0.01),
        Biome::Fungal => (0.02, 0.01, 0.03),
        Biome::CrystalCave => (0.02, 0.03, 0.04),
        Biome::Mountain => (0.02, 0.02, 0.03),
    };

    (
        (r - weather_darken + br).clamp(0.0_f32, 1.0),
        (g - weather_darken + bg).clamp(0.0_f32, 1.0),
        (b - weather_darken + bb).clamp(0.0_f32, 1.0),
    )
}

/// Collects point lights from buildings (campfire, forge) and other emitters.
/// Keeps only the closest MAX_POINT_LIGHTS to the camera for performance.
fn collect_point_lights(
    camera_query: Query<&Transform, With<crate::camera::GameCamera>>,
    building_query: Query<(&Transform, &crate::building::Building)>,
    mut lighting: ResMut<LightingSettings>,
) {
    lighting.point_lights.clear();
    let Ok(cam_tf) = camera_query.get_single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let mut candidates: Vec<(f32, PointLight)> = Vec::new();
    for (tf, building) in building_query.iter() {
        let (color, intensity, radius) = match building.building_type {
            crate::building::BuildingType::Campfire => (
                Vec3::new(1.0, 0.5, 0.2),
                1.2,
                80.0,
            ),
            crate::building::BuildingType::Forge => (
                Vec3::new(0.9, 0.4, 0.1),
                1.0,
                60.0,
            ),
            _ => continue,
        };
        let pos = tf.translation.truncate();
        let dist_sq = cam_pos.distance_squared(pos);
        candidates.push((
            dist_sq,
            PointLight {
                position: pos,
                radius,
                color,
                intensity,
            },
        ));
    }
    candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, light) in candidates.into_iter().take(MAX_POINT_LIGHTS) {
        lighting.point_lights.push(light);
    }
}
