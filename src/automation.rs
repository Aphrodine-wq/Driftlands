use bevy::prelude::*;
use crate::building::{Building, BuildingType, ChestStorage};
use crate::inventory::{InventorySlot, ItemType};
use crate::farming::FarmPlot;
use crate::hud::FloatingTextRequest;
use crate::spatial::SpatialGrid;

pub struct AutomationPlugin;

impl Plugin for AutomationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                auto_smelter_tick,
                crop_sprinkler_tick,
                alarm_bell_tick,
            )
                .run_if(crate::hud::not_paused),
        );
    }
}

// --- Components ---

#[derive(Component)]
pub struct AutoSmelter {
    pub timer: f32,
}

#[derive(Component)]
pub struct CropSprinkler;

#[derive(Component)]
pub struct AlarmBell {
    pub cooldown: f32,
}

// --- Constants ---

/// Auto-smelter converts ore + coal into ingot every 60 seconds.
const SMELTER_INTERVAL: f32 = 60.0;

/// Crop sprinkler waters crops within this pixel radius (2 tiles).
const SPRINKLER_RADIUS: f32 = 32.0;

/// Growth boost multiplier applied per second by sprinkler (25% faster).
const SPRINKLER_BOOST: f32 = 0.25;

/// Alarm bell detection radius in pixels.
const ALARM_RADIUS: f32 = 100.0;

/// Cooldown between alarm bell warnings.
const ALARM_COOLDOWN: f32 = 5.0;

// --- Systems ---

/// Auto-smelter: when placed near a Forge and a Chest containing ore + coal,
/// converts 1 ore + 1 coal into 1 iron ingot every 60 seconds.
fn auto_smelter_tick(
    time: Res<Time>,
    mut smelter_query: Query<(&Transform, &mut AutoSmelter)>,
    forge_query: Query<(&Transform, &Building), Without<AutoSmelter>>,
    mut chest_query: Query<(&Transform, &mut ChestStorage), Without<AutoSmelter>>,
    grid: Res<SpatialGrid>,
) {
    let dt = time.delta_secs();

    for (smelter_tf, mut smelter) in smelter_query.iter_mut() {
        smelter.timer += dt;
        if smelter.timer < SMELTER_INTERVAL {
            continue;
        }
        smelter.timer -= SMELTER_INTERVAL;

        let smelter_pos = smelter_tf.translation.truncate();

        // Use spatial grid to find nearby buildings, then filter for forges
        let nearby_buildings = grid.query_buildings_in_radius(smelter_pos, 64.0);
        let near_forge = nearby_buildings.iter().any(|&(entity, _)| {
            forge_query.get(entity).map(|(_, building)| {
                matches!(building.building_type, BuildingType::Forge | BuildingType::AdvancedForge)
            }).unwrap_or(false)
        });
        if !near_forge {
            continue;
        }

        // Use spatial grid to find nearby buildings, then check chests among them
        for &(chest_entity, _) in &nearby_buildings {
            let Ok((_, mut chest)) = chest_query.get_mut(chest_entity) else { continue };

            // Check if chest has IronOre and Coal
            let has_ore = chest.slots.iter().any(|s| {
                s.as_ref().map(|slot| slot.item == ItemType::IronOre && slot.count >= 1).unwrap_or(false)
            });
            let has_coal = chest.slots.iter().any(|s| {
                s.as_ref().map(|slot| slot.item == ItemType::Coal && slot.count >= 1).unwrap_or(false)
            });

            if !has_ore || !has_coal {
                continue;
            }

            // Remove 1 IronOre
            for slot in chest.slots.iter_mut() {
                if let Some(ref mut s) = slot {
                    if s.item == ItemType::IronOre && s.count >= 1 {
                        s.count -= 1;
                        if s.count == 0 {
                            *slot = None;
                        }
                        break;
                    }
                }
            }

            // Remove 1 Coal
            for slot in chest.slots.iter_mut() {
                if let Some(ref mut s) = slot {
                    if s.item == ItemType::Coal && s.count >= 1 {
                        s.count -= 1;
                        if s.count == 0 {
                            *slot = None;
                        }
                        break;
                    }
                }
            }

            // Add 1 IronIngot to the chest
            // Try stacking first
            let mut added = false;
            for slot in chest.slots.iter_mut() {
                if let Some(ref mut s) = slot {
                    if s.item == ItemType::IronIngot && s.count < 64 {
                        s.count += 1;
                        added = true;
                        break;
                    }
                }
            }
            if !added {
                // Find empty slot
                for slot in chest.slots.iter_mut() {
                    if slot.is_none() {
                        *slot = Some(InventorySlot {
                            item: ItemType::IronIngot,
                            count: 1,
                            durability: None,
                        });
                        break;
                    }
                }
            }

            // Only smelt from one chest per tick
            break;
        }
    }
}

/// Crop sprinkler: auto-waters crops within 2-tile radius, boosting growth by 25%.
fn crop_sprinkler_tick(
    time: Res<Time>,
    sprinkler_query: Query<&Transform, With<CropSprinkler>>,
    mut farm_query: Query<(&Transform, &mut FarmPlot), Without<CropSprinkler>>,
    grid: Res<SpatialGrid>,
) {
    let dt = time.delta_secs();
    let boost = dt * SPRINKLER_BOOST;

    for sprinkler_tf in sprinkler_query.iter() {
        let sprinkler_pos = sprinkler_tf.translation.truncate();

        // Use spatial grid to find nearby farms
        let nearby_farms = grid.query_farms_in_radius(sprinkler_pos, SPRINKLER_RADIUS);
        for (farm_entity, _) in nearby_farms {
            if let Ok((_, mut plot)) = farm_query.get_mut(farm_entity) {
                if plot.crop.is_none() || plot.growth >= 1.0 {
                    continue;
                }
                // Apply 25% growth boost (additive to normal growth each second)
                plot.growth = (plot.growth + boost * (1.0 / 120.0)).min(1.0);
            }
        }
    }
}

/// Alarm bell: alerts when enemies enter 100px radius with floating text.
fn alarm_bell_tick(
    time: Res<Time>,
    mut bell_query: Query<(&Transform, &mut AlarmBell)>,
    mut text_events: EventWriter<FloatingTextRequest>,
    grid: Res<SpatialGrid>,
) {
    let dt = time.delta_secs();

    for (bell_tf, mut bell) in bell_query.iter_mut() {
        // Tick cooldown
        if bell.cooldown > 0.0 {
            bell.cooldown -= dt;
            continue;
        }

        let bell_pos = bell_tf.translation.truncate();

        // Use spatial grid instead of iterating all enemies
        let enemy_nearby = !grid.query_enemies_in_radius(bell_pos, ALARM_RADIUS).is_empty();

        if enemy_nearby {
            text_events.send(FloatingTextRequest {
                text: "!! ALARM: Enemy nearby !!".to_string(),
                position: bell_pos,
                color: Color::srgb(1.0, 0.2, 0.2),
            });
            bell.cooldown = ALARM_COOLDOWN;
        }
    }
}
