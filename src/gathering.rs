use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;
use crate::hud::not_paused;
use crate::world::{WorldObject, WorldObjectType, WorldState};
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::building::BuildingState;
use crate::combat::Enemy;
use crate::particles::SpawnParticlesEvent;
use crate::world::generation::WorldGenerator;
use crate::audio::SoundEvent;
use crate::hud::spawn_floating_text;

pub struct GatheringPlugin;

impl Plugin for GatheringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GatheringState>()
            .add_systems(Update, (
                gather_resources,
                update_gathering_progress_bars,
                cleanup_gathering_visuals,
                pickup_dropped_items,
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
    /// Throttle timer for gather sound events (fires every 0.5s).
    pub sound_timer: f32,
}

/// A progress bar entity that tracks a WorldObject being gathered.
/// Spawned as a separate world-space entity (not a child) so positioning is simple.
#[derive(Component)]
pub struct GatheringProgressBar;

/// A dropped item entity that bobs in the world and gets picked up by the player.
#[derive(Component)]
pub struct DroppedItem {
    pub item: ItemType,
    pub count: u32,
    /// Delay before the item can be picked up (seconds).
    pub spawn_timer: f32,
    /// Timer for the bobbing animation.
    pub bob_timer: f32,
    /// The base Y position (without bob offset applied).
    pub base_y: f32,
}

/// Returns a color for a dropped item based on its type.
fn dropped_item_color(item: ItemType) -> Color {
    match item {
        ItemType::Wood | ItemType::Stick | ItemType::WoodPlank => Color::srgb(0.6, 0.4, 0.2),
        ItemType::Stone | ItemType::StoneBlock => Color::srgb(0.6, 0.6, 0.6),
        ItemType::Flint => Color::srgb(0.45, 0.45, 0.4),
        ItemType::PlantFiber | ItemType::Berry => Color::srgb(0.3, 0.7, 0.2),
        ItemType::IronOre | ItemType::IronIngot => Color::srgb(0.55, 0.45, 0.35),
        ItemType::Coal => Color::srgb(0.15, 0.15, 0.15),
        ItemType::CrystalShard | ItemType::Gemstone => Color::srgb(0.5, 0.4, 0.9),
        ItemType::AncientCore => Color::srgb(0.3, 0.8, 0.7),
        ItemType::IceShard => Color::srgb(0.7, 0.85, 1.0),
        ItemType::MushroomCap | ItemType::Spore => Color::srgb(0.5, 0.3, 0.5),
        ItemType::Sulfur => Color::srgb(0.8, 0.75, 0.2),
        ItemType::CactusFiber => Color::srgb(0.4, 0.6, 0.25),
        ItemType::Reed | ItemType::Peat => Color::srgb(0.4, 0.5, 0.3),
        ItemType::AlpineHerb | ItemType::RareHerb => Color::srgb(0.2, 0.8, 0.4),
        _ => Color::srgb(0.9, 0.85, 0.6), // gold/white fallback
    }
}

/// Spawns a DroppedItem entity at the given world position with a random offset.
fn spawn_dropped_item(commands: &mut Commands, pos: Vec2, item: ItemType, count: u32, rng: &mut impl Rng) {
    let offset = Vec2::new(
        rng.gen_range(-8.0..8.0),
        rng.gen_range(-8.0..8.0),
    );
    let spawn_pos = pos + offset;
    commands.spawn((
        DroppedItem {
            item,
            count,
            spawn_timer: 0.3,
            bob_timer: 0.0,
            base_y: spawn_pos.y,
        },
        Sprite {
            color: dropped_item_color(item),
            custom_size: Some(Vec2::new(6.0, 6.0)),
            ..default()
        },
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 7.0),
    ));
}

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
    mut particle_events: EventWriter<SpawnParticlesEvent>,
    mut sound_events: EventWriter<SoundEvent>,
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

        // Throttled gather sound (every 0.5s)
        gathering_state.sound_timer -= time.delta_secs();
        if gathering_state.sound_timer <= 0.0 {
            gathering_state.sound_timer = 0.5;
            sound_events.send(SoundEvent::Gather);
        }

        if object.health <= 0.0 {
            // Object destroyed — clear target so bars aren't drawn for it
            gathering_state.target = None;

            // Spawn particle effects at destroyed object position
            let obj_pos = obj_transform.translation.truncate();
            let (particle_color, particle_count) = match object.object_type {
                WorldObjectType::OakTree | WorldObjectType::PineTree => {
                    (Color::srgb(0.5, 0.35, 0.2), 4)
                }
                WorldObjectType::Rock | WorldObjectType::IronVein | WorldObjectType::CoalDeposit
                | WorldObjectType::RuinWall => {
                    (Color::srgb(0.6, 0.6, 0.6), 4)
                }
                _ => (Color::srgb(0.2, 0.7, 0.2), 3),
            };
            particle_events.send(SpawnParticlesEvent {
                position: obj_pos,
                color: particle_color,
                count: particle_count,
            });

            // Derive a deterministic hash from world position for rare drops
            let tile_x = (obj_transform.translation.x / 16.0) as i32;
            let tile_y = (obj_transform.translation.y / 16.0) as i32;
            let rare_hash = WorldGenerator::position_hash(tile_x, tile_y, world_state.seed.wrapping_add(99));

            let mut rng = rand::thread_rng();
            match object.object_type {
                WorldObjectType::OakTree | WorldObjectType::PineTree => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Wood, 3, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Stick, 2, &mut rng);
                }
                WorldObjectType::Rock => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Stone, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Flint, 1, &mut rng);
                }
                WorldObjectType::Bush => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::PlantFiber, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Berry, 1, &mut rng);
                }
                WorldObjectType::Cactus => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::CactusFiber, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Stick, 1, &mut rng);
                }
                WorldObjectType::IceCrystal => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::IceShard, 2, &mut rng);
                }
                WorldObjectType::Mushroom => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::MushroomCap, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Spore, 1, &mut rng);
                }
                WorldObjectType::GiantMushroom => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::MushroomCap, 4, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Spore, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Wood, 2, &mut rng);
                }
                WorldObjectType::ReedClump => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Reed, 3, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Peat, 1, &mut rng);
                }
                WorldObjectType::SulfurDeposit => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Sulfur, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Stone, 1, &mut rng);
                }
                WorldObjectType::CrystalNode => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::CrystalShard, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Stone, 1, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Gemstone, 1, &mut rng);
                }
                WorldObjectType::AlpineFlower => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::AlpineHerb, 1, &mut rng);
                    if rare_hash % 100 < 20 {
                        spawn_dropped_item(&mut commands, obj_pos, ItemType::RareHerb, 1, &mut rng);
                    }
                }
                WorldObjectType::IronVein => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::IronOre, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Stone, 1, &mut rng);
                }
                WorldObjectType::CoalDeposit => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Coal, 2, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Stone, 1, &mut rng);
                }
                WorldObjectType::AncientRuin => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::AncientCore, 1, &mut rng);
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::Gemstone, 1, &mut rng);
                }
                WorldObjectType::SupplyCrate => {
                    let supply_roll = WorldGenerator::position_hash(tile_x, tile_y, world_state.seed.wrapping_add(200));
                    match supply_roll % 4 {
                        0 => { spawn_dropped_item(&mut commands, obj_pos, ItemType::Berry, 3, &mut rng); }
                        1 => { spawn_dropped_item(&mut commands, obj_pos, ItemType::Rope, 2, &mut rng); }
                        2 => { spawn_dropped_item(&mut commands, obj_pos, ItemType::Torch, 2, &mut rng); }
                        _ => { spawn_dropped_item(&mut commands, obj_pos, ItemType::Stick, 4, &mut rng); }
                    }
                }
                WorldObjectType::RuinWall => {
                    spawn_dropped_item(&mut commands, obj_pos, ItemType::StoneBlock, 2, &mut rng);
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

/// Animates dropped items (bobbing), attracts them toward the player, and
/// picks them up when close enough.
fn pickup_dropped_items(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut item_query: Query<(Entity, &mut DroppedItem, &mut Transform), Without<Player>>,
    mut inventory: ResMut<Inventory>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_secs();

    for (entity, mut dropped, mut tf) in item_query.iter_mut() {
        // Tick spawn delay
        if dropped.spawn_timer > 0.0 {
            dropped.spawn_timer -= dt;
        }

        // Increment bob timer
        dropped.bob_timer += dt;

        // Bob animation: oscillate Y by +/-2px using absolute offset from base_y
        let bob_offset = (dropped.bob_timer * 3.0 * PI * 2.0).sin() * 2.0;

        // Use base_y for distance calculations (ignore bob offset)
        let item_pos = Vec2::new(tf.translation.x, dropped.base_y);

        // Skip pickup logic if still in spawn delay
        if dropped.spawn_timer > 0.0 {
            tf.translation.y = dropped.base_y + bob_offset;
            continue;
        }

        let dist = item_pos.distance(player_pos);

        // Attract toward player if within 48px
        if dist <= 48.0 && dist > 8.0 {
            let dir = (player_pos - item_pos).normalize_or_zero();
            let move_amount = dir * 200.0 * dt;
            tf.translation.x += move_amount.x;
            dropped.base_y += move_amount.y;
        }

        // Apply bob on top of base_y
        tf.translation.y = dropped.base_y + bob_offset;

        // Pickup if within 8px
        if dist <= 8.0 {
            inventory.add_item(dropped.item, dropped.count);
            sound_events.send(SoundEvent::Pickup);
            // US-028: Floating text for item pickup
            let pickup_text = if dropped.count > 1 {
                format!("+{} {}", dropped.count, dropped.item.display_name())
            } else {
                format!("+1 {}", dropped.item.display_name())
            };
            spawn_floating_text(
                &mut commands,
                &pickup_text,
                Vec2::new(tf.translation.x, dropped.base_y),
                Color::WHITE,
            );
            commands.entity(entity).despawn();
        }
    }
}
