use bevy::prelude::*;
use crate::building::BuildingState;
use crate::hud::not_paused;
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::season::{Season, SeasonCycle};
use crate::weather::{Weather, WeatherSystem};
use crate::world::TILE_SIZE;

pub struct FarmingPlugin;

impl Plugin for FarmingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            till_soil,
            plant_seed,
            grow_crops,
            harvest_crop,
        ).run_if(not_paused));
    }
}

// ── Constants ────────────────────────────────────────────────────────────────

/// Full grow time in real-time seconds (base, before season modifier).
const BASE_GROW_TIME: f32 = 60.0;

/// Interaction range for farming actions (pixels).
const FARM_RANGE: f32 = 32.0;

// ── Components ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CropType {
    Wheat,
    Carrot,
    Tomato,
    Pumpkin,
    Corn,
    Potato,
    Melon,
    Rice,
    Pepper,
    Onion,
    Flax,
    Sugarcane,
}

impl CropType {
    /// The seed item needed to plant this crop.
    pub fn seed_item(&self) -> ItemType {
        match self {
            CropType::Wheat => ItemType::WheatSeed,
            CropType::Carrot => ItemType::CarrotSeed,
            CropType::Tomato => ItemType::TomatoSeed,
            CropType::Pumpkin => ItemType::PumpkinSeed,
            CropType::Corn => ItemType::CornSeed,
            CropType::Potato => ItemType::PotatoSeed,
            CropType::Melon => ItemType::MelonSeed,
            CropType::Rice => ItemType::RiceSeed,
            CropType::Pepper => ItemType::PepperSeed,
            CropType::Onion => ItemType::OnionSeed,
            CropType::Flax => ItemType::FlaxSeed,
            CropType::Sugarcane => ItemType::SugarcaneSeed,
        }
    }

    /// The item yielded on harvest.
    pub fn yield_item(&self) -> ItemType {
        match self {
            CropType::Wheat => ItemType::Wheat,
            CropType::Carrot => ItemType::Carrot,
            CropType::Tomato => ItemType::Tomato,
            CropType::Pumpkin => ItemType::Pumpkin,
            CropType::Corn => ItemType::Corn,
            CropType::Potato => ItemType::Potato,
            CropType::Melon => ItemType::Melon,
            CropType::Rice => ItemType::Rice,
            CropType::Pepper => ItemType::Pepper,
            CropType::Onion => ItemType::Onion,
            CropType::Flax => ItemType::Flax,
            CropType::Sugarcane => ItemType::Sugarcane,
        }
    }

    /// Amount harvested per plot.
    pub fn yield_count(&self) -> u32 {
        match self {
            CropType::Wheat => 3,
            CropType::Carrot => 2,
            CropType::Tomato => 2,
            CropType::Pumpkin => 1,
            CropType::Corn => 3,
            CropType::Potato => 2,
            CropType::Melon => 1,
            CropType::Rice => 3,
            CropType::Pepper => 2,
            CropType::Onion => 2,
            CropType::Flax => 2,
            CropType::Sugarcane => 2,
        }
    }

    pub fn mature_color(&self) -> Color {
        match self {
            CropType::Wheat => Color::srgb(0.9, 0.8, 0.2),
            CropType::Carrot => Color::srgb(0.95, 0.5, 0.1),
            CropType::Tomato => Color::srgb(0.9, 0.2, 0.15),
            CropType::Pumpkin => Color::srgb(0.9, 0.5, 0.1),
            CropType::Corn => Color::srgb(0.95, 0.85, 0.3),
            CropType::Potato => Color::srgb(0.7, 0.55, 0.3),
            CropType::Melon => Color::srgb(0.4, 0.75, 0.35),
            CropType::Rice => Color::srgb(0.9, 0.88, 0.7),
            CropType::Pepper => Color::srgb(0.85, 0.2, 0.1),
            CropType::Onion => Color::srgb(0.8, 0.7, 0.4),
            CropType::Flax => Color::srgb(0.6, 0.7, 0.85),
            CropType::Sugarcane => Color::srgb(0.5, 0.8, 0.3),
        }
    }

    pub fn growing_color(&self) -> Color {
        match self {
            CropType::Wheat => Color::srgb(0.4, 0.75, 0.3),
            CropType::Carrot => Color::srgb(0.3, 0.7, 0.25),
            CropType::Tomato => Color::srgb(0.5, 0.6, 0.2),
            CropType::Pumpkin => Color::srgb(0.4, 0.65, 0.2),
            CropType::Corn => Color::srgb(0.35, 0.7, 0.25),
            CropType::Potato => Color::srgb(0.3, 0.6, 0.2),
            CropType::Melon => Color::srgb(0.3, 0.65, 0.25),
            CropType::Rice => Color::srgb(0.4, 0.7, 0.3),
            CropType::Pepper => Color::srgb(0.35, 0.6, 0.2),
            CropType::Onion => Color::srgb(0.35, 0.65, 0.22),
            CropType::Flax => Color::srgb(0.35, 0.6, 0.3),
            CropType::Sugarcane => Color::srgb(0.3, 0.65, 0.2),
        }
    }

    /// Growth factor for this crop in the given season. 0.0 = off-season (no growth), 1.0 = in-season.
    pub fn growth_factor_in(&self, season: Season) -> f32 {
        match self {
            CropType::Wheat => match season {
                Season::Spring | Season::Summer => 1.0,
                _ => 0.0,
            },
            CropType::Carrot => match season {
                Season::Summer | Season::Autumn => 1.0,
                _ => 0.0,
            },
            CropType::Tomato => match season {
                Season::Summer => 1.0,
                _ => 0.0,
            },
            CropType::Pumpkin => match season {
                Season::Autumn => 1.0,
                _ => 0.0,
            },
            CropType::Corn => match season {
                Season::Spring | Season::Summer => 1.0,
                _ => 0.0,
            },
            CropType::Potato => match season {
                Season::Spring | Season::Autumn => 1.0,
                _ => 0.0,
            },
            CropType::Melon => match season {
                Season::Summer => 1.0,
                _ => 0.0,
            },
            CropType::Rice => match season {
                Season::Spring => 1.0,
                _ => 0.0,
            },
            CropType::Pepper => match season {
                Season::Summer | Season::Autumn => 1.0,
                _ => 0.0,
            },
            CropType::Onion => match season {
                Season::Spring | Season::Autumn => 1.0,
                _ => 0.0,
            },
            CropType::Flax => match season {
                Season::Summer => 1.0,
                _ => 0.0,
            },
            CropType::Sugarcane => match season {
                Season::Summer => 1.0,
                _ => 0.0,
            },
        }
    }
}

/// Represents a tilled tile with optional planted crop.
#[derive(Component)]
pub struct FarmPlot {
    /// None = tilled but empty. Some = planted crop.
    pub crop: Option<CropType>,
    /// Growth progress: 0.0 = just planted, 1.0 = mature.
    pub growth: f32,
}

impl FarmPlot {
    pub fn tilled() -> Self {
        Self { crop: None, growth: 0.0 }
    }

    pub fn is_mature(&self) -> bool {
        self.growth >= 1.0
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Right-click with a Hoe on empty ground to create a FarmPlot.
fn till_soil(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    inventory: Res<Inventory>,
    player_query: Query<&Transform, With<Player>>,
    plot_query: Query<&Transform, With<FarmPlot>>,
) {
    // Build mode consumes right-click, skip farming actions.
    if building_state.active {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    // Must have Hoe selected.
    let Some(slot) = inventory.selected_item() else { return };
    if slot.item != ItemType::Hoe {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Snap to nearest tile grid position in front of player.
    let snapped_x = (player_pos.x / TILE_SIZE).round() * TILE_SIZE;
    let snapped_y = (player_pos.y / TILE_SIZE).round() * TILE_SIZE;
    let target = Vec2::new(snapped_x, snapped_y);

    if player_pos.distance(target) > FARM_RANGE {
        return;
    }

    // Don't till if there is already a plot here.
    for plot_tf in plot_query.iter() {
        let plot_pos = plot_tf.translation.truncate();
        if plot_pos.distance(target) < TILE_SIZE * 0.5 {
            return;
        }
    }

    commands.spawn((
        FarmPlot::tilled(),
        Sprite {
            color: Color::srgb(0.45, 0.28, 0.12),
            custom_size: Some(Vec2::new(TILE_SIZE - 2.0, TILE_SIZE - 2.0)),
            ..default()
        },
        Transform::from_xyz(snapped_x, snapped_y, 1.5),
    ));
}

/// Right-click with a seed item on a tilled empty FarmPlot to plant.
fn plant_seed(
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    player_query: Query<&Transform, With<Player>>,
    mut plot_query: Query<(&Transform, &mut FarmPlot, &mut Sprite)>,
) {
    if building_state.active {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(slot) = inventory.selected_item() else { return };
    let crop_type = match slot.item {
        ItemType::WheatSeed => CropType::Wheat,
        ItemType::CarrotSeed => CropType::Carrot,
        ItemType::TomatoSeed => CropType::Tomato,
        ItemType::PumpkinSeed => CropType::Pumpkin,
        ItemType::CornSeed => CropType::Corn,
        ItemType::PotatoSeed => CropType::Potato,
        ItemType::MelonSeed => CropType::Melon,
        ItemType::RiceSeed => CropType::Rice,
        ItemType::PepperSeed => CropType::Pepper,
        ItemType::OnionSeed => CropType::Onion,
        ItemType::FlaxSeed => CropType::Flax,
        ItemType::SugarcaneSeed => CropType::Sugarcane,
        _ => return,
    };

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest empty tilled plot in range.
    let mut best: Option<(f32, usize)> = None;
    for (idx, (plot_tf, plot, _)) in plot_query.iter().enumerate() {
        if plot.crop.is_some() {
            continue;
        }
        let dist = player_pos.distance(plot_tf.translation.truncate());
        if dist <= FARM_RANGE {
            if best.is_none() || dist < best.unwrap().0 {
                best = Some((dist, idx));
            }
        }
    }

    let Some((_, target_idx)) = best else { return };

    // Consume one seed.
    if !inventory.remove_items(crop_type.seed_item(), 1) {
        return;
    }

    // Plant the crop.
    for (i, (_, mut plot, mut sprite)) in plot_query.iter_mut().enumerate() {
        if i == target_idx {
            plot.crop = Some(crop_type);
            plot.growth = 0.0;
            sprite.color = crop_type.growing_color();
            break;
        }
    }
}

/// Advance growth of all planted farm plots based on elapsed time and season.
fn grow_crops(
    season: Res<SeasonCycle>,
    weather: Res<WeatherSystem>,
    time: Res<Time>,
    mut plot_query: Query<(&mut FarmPlot, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    let drought = (season.current == Season::Summer && weather.current == Weather::Clear)
        .then_some(0.6)
        .unwrap_or(1.0);

    for (mut plot, mut sprite) in plot_query.iter_mut() {
        let Some(crop) = plot.crop else { continue };
        if plot.growth >= 1.0 {
            sprite.color = crop.mature_color();
            continue;
        }

        let in_season = crop.growth_factor_in(season.current);
        let multiplier = if in_season > 0.0 {
            in_season * season.current.growth_multiplier() * drought
        } else {
            0.0
        };
        plot.growth += (dt / BASE_GROW_TIME) * multiplier;
        plot.growth = plot.growth.min(1.0);

        // Update visual: blend from growing to mature colour as progress increases.
        if plot.growth >= 1.0 {
            sprite.color = crop.mature_color();
        } else {
            // Simple lerp between growing green and mature colour.
            let g = crop.growing_color();
            let m = crop.mature_color();
            let t = plot.growth;
            sprite.color = Color::srgb(
                g.to_srgba().red   * (1.0 - t) + m.to_srgba().red   * t,
                g.to_srgba().green * (1.0 - t) + m.to_srgba().green * t,
                g.to_srgba().blue  * (1.0 - t) + m.to_srgba().blue  * t,
            );
        }
    }
}

/// Left-click a mature FarmPlot to harvest it; the plot returns to tilled state.
fn harvest_crop(
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    player_query: Query<&Transform, With<Player>>,
    mut plot_query: Query<(&Transform, &mut FarmPlot, &mut Sprite)>,
) {
    if building_state.active {
        return;
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find nearest mature plot in range.
    let mut best: Option<(f32, usize)> = None;
    for (idx, (plot_tf, plot, _)) in plot_query.iter().enumerate() {
        if !plot.is_mature() || plot.crop.is_none() {
            continue;
        }
        let dist = player_pos.distance(plot_tf.translation.truncate());
        if dist <= FARM_RANGE {
            if best.is_none() || dist < best.unwrap().0 {
                best = Some((dist, idx));
            }
        }
    }

    let Some((_, target_idx)) = best else { return };

    for (i, (_, mut plot, mut sprite)) in plot_query.iter_mut().enumerate() {
        if i == target_idx {
            let crop = plot.crop.unwrap();
            inventory.add_item(crop.yield_item(), crop.yield_count());
            // Reset to tilled state.
            plot.crop = None;
            plot.growth = 0.0;
            sprite.color = Color::srgb(0.45, 0.28, 0.12);
            break;
        }
    }
}
