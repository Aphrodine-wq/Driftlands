use bevy::prelude::*;
use crate::hud::not_paused;
use crate::world::{WorldObject, WorldObjectType, WorldState};
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::building::BuildingState;
use crate::combat::Enemy;
use crate::world::generation::WorldGenerator;

pub struct GatheringPlugin;

impl Plugin for GatheringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GatheringState>()
            .add_systems(Update, (
                gather_resources,
                update_gathering_progress_bars,
                cleanup_gathering_visuals,
            ).chain().run_if(not_paused));
    }
}

const GATHER_RANGE: f32 = 32.0;

/// Tracks which WorldObject the player is currently gathering (if any).
/// Uses a Resource instead of a marker Component so that the value is
/// available to later systems in the same frame without waiting for
/// command buffer application.
#[derive(Resource, Default)]
pub struct GatheringState {
    pub target: Option<Entity>,
}

/// A progress bar entity that tracks a WorldObject being gathered.
/// Spawned as a separate world-space entity (not a child) so positioning is simple.
#[derive(Component)]
pub struct GatheringProgressBar;

fn gather_resources(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    player_query: Query<&Transform, With<Player>>,
    mut objects_query: Query<(Entity, &Transform, &mut WorldObject)>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut inventory: ResMut<Inventory>,
    world_state: Res<WorldState>,
    time: Res<Time>,
    mut gathering_state: ResMut<GatheringState>,
) {
    // Default: not gathering anything this frame
    gathering_state.target = None;

    if !mouse.pressed(MouseButton::Left) || building_state.active {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else { return };
    let player_pos = player_transform.translation.truncate();

    // Don't gather if an enemy is within attack range (combat takes priority)
    for enemy_tf in enemy_query.iter() {
        if player_pos.distance(enemy_tf.translation.truncate()) <= 40.0 {
            return;
        }
    }

    // Find nearest object in range
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, transform, _) in objects_query.iter() {
        let dist = player_pos.distance(transform.translation.truncate());
        if dist <= GATHER_RANGE {
            if nearest.is_none() || dist < nearest.unwrap().1 {
                nearest = Some((entity, dist));
            }
        }
    }

    let Some((target_entity, _)) = nearest else { return };

    if let Ok((_, obj_transform, mut object)) = objects_query.get_mut(target_entity) {
        // Check tool tier requirement
        let required_tier = object.object_type.min_tool_tier();
        if required_tier > 0 {
            let player_tool_tier = inventory.selected_item()
                .map(|s| s.item.tool_tier())
                .unwrap_or(0);
            if player_tool_tier < required_tier {
                return; // Need better tool
            }
        }

        let tool_bonus = if required_tier > 0 {
            let tier = inventory.selected_item().map(|s| s.item.tool_tier()).unwrap_or(0);
            if tier > required_tier { 1.5 } else { 1.0 }
        } else { 1.0 };

        object.health -= 30.0 * tool_bonus * time.delta_secs();

        // Record current gathering target for visual feedback systems
        gathering_state.target = Some(target_entity);

        if object.health <= 0.0 {
            // Object destroyed — clear target so bars aren't drawn for it
            gathering_state.target = None;

            // Derive a deterministic hash from world position for rare drops
            let tile_x = (obj_transform.translation.x / 16.0) as i32;
            let tile_y = (obj_transform.translation.y / 16.0) as i32;
            let rare_hash = WorldGenerator::position_hash(tile_x, tile_y, world_state.seed.wrapping_add(99));

            match object.object_type {
                WorldObjectType::OakTree | WorldObjectType::PineTree => {
                    inventory.add_item(ItemType::Wood, 3);
                    inventory.add_item(ItemType::Stick, 2);
                }
                WorldObjectType::Rock => {
                    inventory.add_item(ItemType::Stone, 2);
                    inventory.add_item(ItemType::Flint, 1);
                }
                WorldObjectType::Bush => {
                    inventory.add_item(ItemType::PlantFiber, 2);
                    inventory.add_item(ItemType::Berry, 1);
                }
                WorldObjectType::Cactus => {
                    inventory.add_item(ItemType::CactusFiber, 2);
                    inventory.add_item(ItemType::Stick, 1);
                }
                WorldObjectType::IceCrystal => {
                    inventory.add_item(ItemType::IceShard, 2);
                }
                WorldObjectType::Mushroom => {
                    inventory.add_item(ItemType::MushroomCap, 2);
                    inventory.add_item(ItemType::Spore, 1);
                }
                WorldObjectType::GiantMushroom => {
                    inventory.add_item(ItemType::MushroomCap, 4);
                    inventory.add_item(ItemType::Spore, 2);
                    inventory.add_item(ItemType::Wood, 2);
                }
                WorldObjectType::ReedClump => {
                    inventory.add_item(ItemType::Reed, 3);
                    inventory.add_item(ItemType::Peat, 1);
                }
                WorldObjectType::SulfurDeposit => {
                    inventory.add_item(ItemType::Sulfur, 2);
                    inventory.add_item(ItemType::Stone, 1);
                }
                WorldObjectType::CrystalNode => {
                    inventory.add_item(ItemType::CrystalShard, 2);
                    inventory.add_item(ItemType::Stone, 1);
                    // Always drop 1 Gemstone
                    inventory.add_item(ItemType::Gemstone, 1);
                }
                WorldObjectType::AlpineFlower => {
                    inventory.add_item(ItemType::AlpineHerb, 1);
                    // Rare drop: RareHerb on ~20% of harvests
                    if rare_hash % 100 < 20 {
                        inventory.add_item(ItemType::RareHerb, 1);
                    }
                }
                WorldObjectType::IronVein => {
                    inventory.add_item(ItemType::IronOre, 2);
                    inventory.add_item(ItemType::Stone, 1);
                }
                WorldObjectType::CoalDeposit => {
                    inventory.add_item(ItemType::Coal, 2);
                    inventory.add_item(ItemType::Stone, 1);
                }
                WorldObjectType::AncientRuin => {
                    inventory.add_item(ItemType::AncientCore, 1);
                    inventory.add_item(ItemType::Gemstone, 1);
                }
                WorldObjectType::SupplyCrate => {
                    // Random supplies
                    let supply_roll = WorldGenerator::position_hash(tile_x, tile_y, world_state.seed.wrapping_add(200));
                    match supply_roll % 4 {
                        0 => { inventory.add_item(ItemType::Berry, 3); }
                        1 => { inventory.add_item(ItemType::Rope, 2); }
                        2 => { inventory.add_item(ItemType::Torch, 2); }
                        _ => { inventory.add_item(ItemType::Stick, 4); }
                    }
                }
                WorldObjectType::RuinWall => {
                    inventory.add_item(ItemType::StoneBlock, 2);
                }
            }
            // Consume tool durability
            inventory.use_selected_tool();
            commands.entity(target_entity).despawn();
        }
    }
}

/// Spawns, updates, and positions gathering progress bar entities.
/// Follows the same despawn-and-recreate pattern used by EnemyHealthBar in combat.rs.
fn update_gathering_progress_bars(
    mut commands: Commands,
    bar_query: Query<Entity, With<GatheringProgressBar>>,
    gathering_state: Res<GatheringState>,
    object_query: Query<(&Transform, &WorldObject)>,
) {
    // Despawn all existing progress bar entities each frame
    for entity in bar_query.iter() {
        commands.entity(entity).despawn();
    }

    // Only draw bars if we're actively gathering something
    let Some(target_entity) = gathering_state.target else { return };
    let Ok((tf, object)) = object_query.get(target_entity) else { return };

    let max_health = object.object_type.max_health();
    let ratio = (object.health / max_health).clamp(0.0, 1.0);

    let bar_width = 24.0;
    let bar_height = 4.0;
    let bar_y = tf.translation.y + 12.0;

    // Background bar (dark gray)
    commands.spawn((
        GatheringProgressBar,
        Sprite {
            color: Color::srgb(0.2, 0.2, 0.2),
            custom_size: Some(Vec2::new(bar_width, bar_height)),
            ..default()
        },
        Transform::from_xyz(tf.translation.x, bar_y, 9.0),
    ));

    // Fill bar (green), shrinks from left as health decreases
    let fill_width = bar_width * ratio;
    let fill_offset = (bar_width - fill_width) / 2.0;
    commands.spawn((
        GatheringProgressBar,
        Sprite {
            color: Color::srgb(0.2, 0.8, 0.2),
            custom_size: Some(Vec2::new(fill_width, bar_height)),
            ..default()
        },
        Transform::from_xyz(tf.translation.x - fill_offset, bar_y, 9.1),
    ));
}

/// Applies visual scale feedback to objects being gathered, and resets scale
/// when gathering stops.
fn cleanup_gathering_visuals(
    gathering_state: Res<GatheringState>,
    mut object_query: Query<(Entity, &WorldObject, &mut Transform)>,
) {
    for (entity, object, mut tf) in object_query.iter_mut() {
        if gathering_state.target == Some(entity) {
            // Scale down based on remaining health ratio
            let max_health = object.object_type.max_health();
            let ratio = (object.health / max_health).clamp(0.0, 1.0);
            tf.scale = Vec3::splat(0.7 + 0.3 * ratio);
        } else if tf.scale != Vec3::ONE {
            // Reset scale on objects no longer being gathered
            tf.scale = Vec3::ONE;
        }
    }
}
