use bevy::prelude::*;
use crate::camera::GameCamera;
use crate::hud::{not_paused, CurrentBiome};
use crate::player::{Player, Health};
use crate::weather::{Weather, WeatherSystem};
use crate::world::generation::Biome;
use crate::season::Season;

pub struct DayNightPlugin;

impl Plugin for DayNightPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DayNightCycle::default())
            .add_systems(Startup, spawn_overlay)
            .add_systems(Update, (update_day_night, apply_ambient_light).run_if(not_paused))
            // Ambient light color updates even when paused (purely visual)
            .add_systems(Update, update_clear_color);
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
        self.phase_with_season(Season::Summer)
    }

    /// Winter has longer nights (PRD 5.6): night 0–0.25 and 0.75–1.0; day 0.35–0.65.
    pub fn phase_with_season(&self, season: Season) -> DayPhase {
        let (night_end, day_start, day_end, sunset_start) = if season == Season::Winter {
            (0.25, 0.35, 0.65, 0.75)
        } else {
            (0.2, 0.3, 0.7, 0.8)
        };
        let t = self.time_of_day;
        if t < night_end || t >= sunset_start {
            DayPhase::Night
        } else if t < day_start {
            DayPhase::Sunrise
        } else if t < day_end {
            DayPhase::Day
        } else {
            DayPhase::Sunset
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

#[derive(Component)]
pub struct VignetteOverlay;

fn spawn_overlay(mut commands: Commands, assets: Res<crate::assets::GameAssets>) {
    // Day/night darkness overlay
    commands.spawn((
        DayNightOverlay,
        Sprite {
            color: Color::srgba(0.05, 0.05, 0.2, 0.0),
            custom_size: Some(Vec2::new(4096.0, 4096.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 50.0),
    ));

    // Vignette overlay — always present, subtle edge darkening
    commands.spawn((
        VignetteOverlay,
        Sprite {
            image: assets.vignette.clone(),
            custom_size: Some(Vec2::new(2048.0, 2048.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 49.0),
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
    season: Res<crate::season::SeasonCycle>,
    weather: Res<WeatherSystem>,
    current_biome: Res<CurrentBiome>,
    player_query: Query<&Health, With<Player>>,
    mut overlay_query: Query<(&mut Sprite, &mut Transform), (With<DayNightOverlay>, Without<GameCamera>, Without<VignetteOverlay>)>,
    mut vignette_query: Query<(&mut Sprite, &mut Transform), (With<VignetteOverlay>, Without<GameCamera>, Without<DayNightOverlay>)>,
    camera_query: Query<&Transform, (With<GameCamera>, Without<DayNightOverlay>, Without<VignetteOverlay>)>,
) {
    let Ok((mut sprite, mut overlay_tf)) = overlay_query.get_single_mut() else { return };
    let Ok(cam_tf) = camera_query.get_single() else { return };

    // Follow camera so overlays always cover the screen
    overlay_tf.translation.x = cam_tf.translation.x;
    overlay_tf.translation.y = cam_tf.translation.y;

    // Vignette follows camera too
    if let Ok((mut vig_sprite, mut vig_tf)) = vignette_query.get_single_mut() {
        vig_tf.translation.x = cam_tf.translation.x;
        vig_tf.translation.y = cam_tf.translation.y;

        // --- Screen-space mood: vignette intensity & tint by time, weather, biome, and health ---
        let health_frac = player_query.get_single().ok()
            .map(|h| (h.current / h.max).clamp(0.0, 1.0))
            .unwrap_or(1.0);

        // Reduced now that per-pixel lighting carries mood; vignette is subtle frame only
        let base_alpha = match cycle.phase_with_season(season.current) {
            DayPhase::Day => 0.10,
            DayPhase::Sunrise | DayPhase::Sunset => 0.14,
            DayPhase::Night => 0.18,
        };

        let weather_boost = match weather.current {
            Weather::Clear => 0.0,
            Weather::Rain | Weather::Snow => 0.03,
            Weather::Storm => 0.07,
            Weather::Fog => 0.05,
            Weather::Blizzard => 0.08,
        };

        // Lower health -> stronger vignette + red tint
        let low_health = (1.0 - health_frac).clamp(0.0, 1.0);
        let health_boost = 0.10 * low_health;

        let alpha = (base_alpha + weather_boost + health_boost).clamp(0.05, 0.40);

        // Subtle biome tint
        let (mut r, mut g, mut b, _) = match current_biome.biome.unwrap_or(Biome::Forest) {
            Biome::Forest => (0.05_f32, 0.08_f32, 0.03_f32, 1.0_f32),
            Biome::Coastal => (0.03_f32, 0.08_f32, 0.10_f32, 1.0_f32),
            Biome::Swamp => (0.04_f32, 0.06_f32, 0.03_f32, 1.0_f32),
            Biome::Desert => (0.08_f32, 0.06_f32, 0.02_f32, 1.0_f32),
            Biome::Tundra => (0.05_f32, 0.07_f32, 0.10_f32, 1.0_f32),
            Biome::Volcanic => (0.08_f32, 0.03_f32, 0.02_f32, 1.0_f32),
            Biome::Fungal => (0.05_f32, 0.03_f32, 0.07_f32, 1.0_f32),
            Biome::CrystalCave => (0.04_f32, 0.06_f32, 0.09_f32, 1.0_f32),
            Biome::Mountain => (0.05_f32, 0.06_f32, 0.07_f32, 1.0_f32),
        };

        // Health-based red accent: only really visible at low HP
        let red_accent = 0.08 * low_health;
        r = (r + red_accent).clamp(0.0, 0.3);
        g = g.clamp(0.0, 0.3);
        b = b.clamp(0.0, 0.3);

        vig_sprite.color = Color::srgba(r, g, b, alpha);
    }

    // Lighter overlay so per-pixel chunk lighting is the main day/night driver
    let darkness = match cycle.phase_with_season(season.current) {
        DayPhase::Night => 0.32,
        DayPhase::Sunrise => {
            let f = (cycle.time_of_day - 0.2) / 0.1;
            0.32 * (1.0 - f)
        }
        DayPhase::Day => 0.0,
        DayPhase::Sunset => {
            let f = (cycle.time_of_day - 0.7) / 0.1;
            0.32 * f
        }
    };

    sprite.color = Color::srgba(0.02, 0.02, 0.10, darkness);
}

/// Smoothly shifts ClearColor between day/night atmosphere colors.
fn update_clear_color(
    cycle: Res<DayNightCycle>,
    mut clear_color: ResMut<ClearColor>,
) {
    let t = cycle.time_of_day;

    // Key colors for each phase
    let midnight = Color::srgb(0.02, 0.02, 0.06);
    let dawn = Color::srgb(0.15, 0.08, 0.05);
    let day = Color::srgb(0.05, 0.05, 0.1); // Keep dark — background behind tiles
    let dusk = Color::srgb(0.12, 0.05, 0.08);

    // Lerp between phases
    let color = if t < 0.2 {
        // Night (midnight to pre-dawn)
        midnight
    } else if t < 0.3 {
        // Dawn transition
        let frac = ((t - 0.2) / 0.1).clamp(0.0, 1.0);
        lerp_color(midnight, dawn, frac)
    } else if t < 0.4 {
        // Dawn to day
        let frac = ((t - 0.3) / 0.1).clamp(0.0, 1.0);
        lerp_color(dawn, day, frac)
    } else if t < 0.7 {
        // Daytime
        day
    } else if t < 0.8 {
        // Dusk transition
        let frac = ((t - 0.7) / 0.1).clamp(0.0, 1.0);
        lerp_color(day, dusk, frac)
    } else if t < 0.9 {
        // Dusk to night
        let frac = ((t - 0.8) / 0.1).clamp(0.0, 1.0);
        lerp_color(dusk, midnight, frac)
    } else {
        // Deep night
        midnight
    };

    clear_color.0 = color;
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgb(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
    )
}
