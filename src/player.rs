use bevy::prelude::*;
use crate::building::{Building, BuildingState, BuildingType, Door};
use crate::hud::not_paused;
use crate::inventory::{Inventory, ItemType};
use crate::world::{TILE_SIZE, CHUNK_WORLD_SIZE};
use crate::world::chunk::{Chunk, CHUNK_SIZE};
use crate::audio::SoundEvent;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ArmorSlots::default())
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (
                player_movement,
                hunger_depletion,
                starvation_damage,
                health_regeneration,
                eat_food,
                buff_tick,
                equip_armor,
                update_damage_flash,
                update_attack_lunge,
            ).run_if(not_paused));
    }
}

/// Which floor the player is on (0 = ground, 1 = first floor). Used for building placement and visibility.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CurrentFloor(pub u8);

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub walk_timer: f32,
    pub velocity: Vec2,
    /// Tracks the current Y offset applied by the walk hop so it can be reversed.
    pub walk_bob_offset: f32,
}

/// Flashes the player sprite red when they take damage.
#[derive(Component)]
pub struct DamageFlash {
    pub timer: f32,
}

/// Briefly scales the player sprite up on melee attack for a lunge effect.
#[derive(Component)]
pub struct AttackLunge {
    pub timer: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum PlayerFacing {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

#[derive(Component)]
pub struct Hunger {
    pub current: f32,
    pub max: f32,
    pub starvation_timer: f32,
}

impl Hunger {
    pub fn new(max: f32) -> Self {
        Self { current: max, max, starvation_timer: 0.0 }
    }

    pub fn eat(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn is_starving(&self) -> bool {
        self.current <= 0.0
    }

    pub fn is_slow(&self) -> bool {
        self.current < self.max * 0.2
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuffType {
    Speed,
    Strength,
    Regen,
}

/// A temporary buff applied to the player from a potion.
#[derive(Component)]
pub struct ActiveBuff {
    pub buff_type: BuffType,
    /// Remaining duration in seconds.
    pub remaining: f32,
    /// Multiplicative magnitude (e.g. 1.5 = 50% bonus).
    pub magnitude: f32,
}

pub const PLAYER_SPEED: f32 = 150.0;
const PLAYER_SIZE: f32 = 20.0;

// Acceleration / deceleration constants for smooth movement
const ACCEL: f32 = 800.0;   // pixels/sec^2 — reach max speed in ~0.19s
const DECEL: f32 = 1200.0;  // friction when no input — stop in ~0.13s

fn spawn_player(
    mut commands: Commands,
    assets: Res<crate::assets::GameAssets>,
    world_state: Res<crate::world::WorldState>,
) {
    // Find a walkable, non-water spawn point near the default position
    let gen = &world_state.generator;
    let mut spawn_x = TILE_SIZE * 16.0;
    let mut spawn_y = TILE_SIZE * 16.0;

    // Search in expanding rings for a valid spawn tile
    'search: for radius in 0i32..20 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius { continue; }
                let wx = (16 + dx) as f64;
                let wy = (16 + dy) as f64;
                let biome = gen.biome_at(wx, wy);
                // Avoid spawning in water-heavy biomes at the exact tile
                let is_water_biome = matches!(biome, crate::world::generation::Biome::Coastal | crate::world::generation::Biome::Swamp);
                if !is_water_biome || radius > 5 {
                    spawn_x = wx as f32 * TILE_SIZE;
                    spawn_y = wy as f32 * TILE_SIZE;
                    break 'search;
                }
            }
        }
    }

    commands.spawn((
        Player { speed: PLAYER_SPEED, walk_timer: 0.0, velocity: Vec2::ZERO, walk_bob_offset: 0.0 },
        CurrentFloor(0),
        PlayerFacing::Down,
        Health::new(100.0),
        Hunger::new(100.0),
        Sprite {
            image: assets.player.clone(),
            custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(spawn_x, spawn_y, 10.0),
    ));
}

fn player_movement(
    mut query: Query<(&mut Player, &Hunger, &mut Transform, &mut PlayerFacing, &mut Sprite, Option<&DamageFlash>, Option<&crate::combat::VineRootTrap>)>,
    buffs_query: Query<&ActiveBuff>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    chunk_query: Query<&Chunk>,
    building_query: Query<(&Transform, &Building, Option<&Door>), Without<Player>>,
    game_settings: Res<crate::settings::GameSettings>,
) {
    let Ok((mut player, hunger, mut transform, mut facing, mut sprite, damage_flash, vine_trap)) = query.get_single_mut() else { return };
    let dt = time.delta_secs();

    // Vine root trap: freeze movement while rooted
    if vine_trap.is_some() {
        player.velocity = Vec2::ZERO;
        return;
    }

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(game_settings.keybinds.move_up) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(game_settings.keybinds.move_down) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(game_settings.keybinds.move_left) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(game_settings.keybinds.move_right) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Compute max speed with buffs/hunger
    let hunger_multiplier = if hunger.is_slow() { 0.7 } else { 1.0 };
    let speed_buff = buffs_query.iter()
        .find(|b| b.buff_type == BuffType::Speed)
        .map(|b| b.magnitude)
        .unwrap_or(1.0);
    // Swimming: 50% speed in water, visual tint
    let in_water = is_position_water(transform.translation.x, transform.translation.y, &chunk_query);
    let swim_mult = if in_water { 0.5 } else { 1.0 };
    let max_speed = player.speed * hunger_multiplier * speed_buff * swim_mult;

    // Swim visual: blue tint when in water (skip if damage flash is active)
    if damage_flash.is_none() {
        if in_water {
            sprite.color = Color::srgba(0.7, 0.8, 1.0, 0.85);
        } else {
            sprite.color = Color::WHITE;
        }
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();

        // Determine facing from the largest axis of input
        if direction.x.abs() >= direction.y.abs() {
            if direction.x > 0.0 {
                *facing = PlayerFacing::Right;
                sprite.flip_x = false;
            } else {
                *facing = PlayerFacing::Left;
                sprite.flip_x = true;
            }
        } else if direction.y > 0.0 {
            *facing = PlayerFacing::Up;
        } else {
            *facing = PlayerFacing::Down;
        }

        // Accelerate toward input direction
        player.velocity += direction * ACCEL * dt;
        // Clamp to max speed
        let speed_sq = player.velocity.length_squared();
        if speed_sq > max_speed * max_speed {
            player.velocity = player.velocity.normalize() * max_speed;
        }
    } else {
        // Decelerate toward zero (friction)
        let current_speed = player.velocity.length();
        if current_speed > 0.0 {
            let reduction = DECEL * dt;
            if reduction >= current_speed {
                player.velocity = Vec2::ZERO;
            } else {
                let dir = player.velocity.normalize();
                player.velocity -= dir * reduction;
            }
        }
    }

    // Apply velocity with per-axis collision
    let delta = player.velocity * dt;

    if delta.x != 0.0 {
        let target_x = transform.translation.x + delta.x;
        if is_position_walkable(target_x, transform.translation.y, &chunk_query)
            && !is_blocked_by_building(target_x, transform.translation.y, &building_query)
        {
            transform.translation.x = target_x;
        } else {
            // Zero out X velocity on collision
            player.velocity.x = 0.0;
        }
    }

    if delta.y != 0.0 {
        let target_y = transform.translation.y + delta.y;
        if is_position_walkable(transform.translation.x, target_y, &chunk_query)
            && !is_blocked_by_building(transform.translation.x, target_y, &building_query)
        {
            transform.translation.y = target_y;
        } else {
            // Zero out Y velocity on collision
            player.velocity.y = 0.0;
        }
    }

    // Walk bob: pronounced squash/stretch + hop when moving
    let is_moving = player.velocity.length_squared() > 1.0;
    // Remove previous hop offset so position stays correct
    transform.translation.y -= player.walk_bob_offset;
    if is_moving {
        player.walk_timer += dt;
        let bob = (player.walk_timer * 10.0 * std::f32::consts::PI).sin();
        sprite.custom_size = Some(Vec2::new(
            PLAYER_SIZE + bob * 1.5,
            PLAYER_SIZE - bob * 1.5,
        ));
        // Hop effect: visually bounce the sprite upward
        let hop = bob.abs() * 0.8;
        player.walk_bob_offset = hop;
        transform.translation.y += hop;
    } else {
        player.walk_timer = 0.0;
        player.walk_bob_offset = 0.0;
        sprite.custom_size = Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE));
    }
}

fn is_position_walkable(x: f32, y: f32, chunk_query: &Query<&Chunk>) -> bool {
    let chunk_x = (x / CHUNK_WORLD_SIZE).floor() as i32;
    let chunk_y = (y / CHUNK_WORLD_SIZE).floor() as i32;

    let tile_x = ((x / TILE_SIZE).floor() as i32 - chunk_x * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
    let tile_y = ((y / TILE_SIZE).floor() as i32 - chunk_y * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;

    for chunk in chunk_query.iter() {
        if chunk.position.x == chunk_x && chunk.position.y == chunk_y {
            return chunk.get_tile(tile_x, tile_y).is_walkable();
        }
    }

    true // Allow movement if chunk not loaded
}

/// Returns true if the given position is on a water tile.
fn is_position_water(x: f32, y: f32, chunk_query: &Query<&Chunk>) -> bool {
    let chunk_x = (x / CHUNK_WORLD_SIZE).floor() as i32;
    let chunk_y = (y / CHUNK_WORLD_SIZE).floor() as i32;

    let tile_x = ((x / TILE_SIZE).floor() as i32 - chunk_x * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;
    let tile_y = ((y / TILE_SIZE).floor() as i32 - chunk_y * CHUNK_SIZE as i32).clamp(0, CHUNK_SIZE as i32 - 1) as usize;

    for chunk in chunk_query.iter() {
        if chunk.position.x == chunk_x && chunk.position.y == chunk_y {
            return chunk.get_tile(tile_x, tile_y).is_water();
        }
    }

    false
}

fn is_blocked_by_building(
    x: f32,
    y: f32,
    building_query: &Query<(&Transform, &Building, Option<&Door>), Without<Player>>,
) -> bool {
    let player_half = 8.0; // Half of collision box (~80% of PLAYER_SIZE 20)
    for (tf, building, door) in building_query.iter() {
        // Only walls and closed doors block movement
        let blocks = match building.building_type {
            BuildingType::WoodWall | BuildingType::StoneWall | BuildingType::MetalWall | BuildingType::WoodFence => true,
            BuildingType::WoodDoor | BuildingType::StoneDoor | BuildingType::MetalDoor => {
                door.map(|d| !d.is_open).unwrap_or(true)
            }
            _ => false,
        };
        if !blocks {
            continue;
        }

        let bpos = tf.translation.truncate();
        let bsize = building.building_type.size();
        let half_w = bsize.x / 2.0;
        let half_h = bsize.y / 2.0;

        // AABB overlap check
        if x + player_half > bpos.x - half_w
            && x - player_half < bpos.x + half_w
            && y + player_half > bpos.y - half_h
            && y - player_half < bpos.y + half_h
        {
            return true;
        }
    }
    false
}

fn hunger_depletion(
    mut query: Query<&mut Hunger, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut hunger) = query.get_single_mut() else { return };
    // 1.0 hunger per 30 seconds = 1/30 per second
    hunger.current = (hunger.current - time.delta_secs() / 30.0).max(0.0);
}

fn starvation_damage(
    mut query: Query<(&mut Health, &mut Hunger), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut health, mut hunger)) = query.get_single_mut() else { return };
    if hunger.is_starving() && !health.is_dead() {
        hunger.starvation_timer += time.delta_secs();
        if hunger.starvation_timer >= 10.0 {
            hunger.starvation_timer -= 10.0;
            health.take_damage(1.0);
        }
    } else {
        hunger.starvation_timer = 0.0;
    }
}

fn health_regeneration(
    mut query: Query<(&mut Health, &Hunger), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut health, hunger)) = query.get_single_mut() else { return };
    // Only regen when hunger > 50%
    if !health.is_dead() && health.current < health.max && hunger.current > hunger.max * 0.5 {
        health.heal(1.0 * time.delta_secs());
    }
}

fn eat_food(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    mut health_query: Query<(Entity, &mut Health, &mut Hunger), With<Player>>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    if !mouse.just_pressed(MouseButton::Right) || building_state.active {
        return;
    }
    let Ok((player_entity, mut health, mut hunger)) = health_query.get_single_mut() else { return };

    let Some(slot) = inventory.selected_item() else { return };
    let item = slot.item;

    // Map each food item to hunger restored.
    let food_value: Option<f32> = match item {
        ItemType::Berry => Some(15.0),
        ItemType::Wheat => Some(10.0),
        ItemType::Carrot => Some(12.0),
        ItemType::Tomato => Some(8.0),
        ItemType::Pumpkin => Some(14.0),
        ItemType::Corn => Some(10.0),
        ItemType::Potato => Some(12.0),
        ItemType::Melon => Some(8.0),
        ItemType::Rice => Some(8.0),
        ItemType::Pepper => Some(6.0),
        ItemType::Onion => Some(6.0),
        ItemType::CookedBerry => Some(25.0),
        ItemType::BakedWheat => Some(30.0),
        ItemType::CookedCarrot => Some(28.0),
        ItemType::CookedTomato => Some(22.0),
        ItemType::BakedPumpkin => Some(35.0),
        ItemType::RoastedCorn => Some(28.0),
        ItemType::BakedPotato => Some(32.0),
        ItemType::MelonSlice => Some(20.0),
        ItemType::CookedRice => Some(26.0),
        ItemType::RoastedPepper => Some(22.0),
        ItemType::CookedOnion => Some(18.0),
        // Fish
        ItemType::RawTrout => Some(10.0),
        ItemType::RawSalmon => Some(12.0),
        ItemType::RawCatfish => Some(8.0),
        ItemType::RawEel => Some(6.0),
        ItemType::RawCrab => Some(8.0),
        ItemType::CookedTrout => Some(28.0),
        ItemType::CookedSalmon => Some(32.0),
        ItemType::CookedCatfish => Some(24.0),
        ItemType::CookedEel => Some(22.0),
        ItemType::CrabMeat => Some(26.0),
        _ => None,
    };

    if let Some(value) = food_value {
        if inventory.remove_items(item, 1) {
            hunger.eat(value);
            sound_events.send(SoundEvent::Eat);
            // US-039: Cooked foods give temporary buffs
            match item {
                ItemType::CookedBerry => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Speed,
                        remaining: 60.0,
                        magnitude: 1.1,
                    });
                }
                ItemType::BakedWheat => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Strength,
                        remaining: 60.0,
                        magnitude: 1.15,
                    });
                }
                ItemType::CookedCarrot => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Regen,
                        remaining: 90.0,
                        magnitude: 0.2, // 0.2 HP per second = 2 HP per 10s
                    });
                }
                ItemType::CookedTomato | ItemType::BakedPumpkin => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Speed,
                        remaining: 45.0,
                        magnitude: 1.1,
                    });
                }
                ItemType::RoastedCorn | ItemType::BakedPotato => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Strength,
                        remaining: 60.0,
                        magnitude: 1.1,
                    });
                }
                ItemType::CookedRice | ItemType::CookedOnion => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Regen,
                        remaining: 60.0,
                        magnitude: 0.3,
                    });
                }
                ItemType::RoastedPepper => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Speed,
                        remaining: 30.0,
                        magnitude: 1.15,
                    });
                }
                ItemType::MelonSlice => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Regen,
                        remaining: 45.0,
                        magnitude: 0.2,
                    });
                }
                ItemType::CookedSalmon | ItemType::CookedTrout => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Strength,
                        remaining: 45.0,
                        magnitude: 1.1,
                    });
                }
                ItemType::CrabMeat => {
                    commands.entity(player_entity).insert(ActiveBuff {
                        buff_type: BuffType::Speed,
                        remaining: 30.0,
                        magnitude: 1.1,
                    });
                }
                _ => {}
            }
        }
        return;
    }

    // Handle potions
    match item {
        ItemType::HealthPotion => {
            if inventory.remove_items(item, 1) {
                health.heal(50.0);
            }
        }
        ItemType::SpeedPotion => {
            if inventory.remove_items(item, 1) {
                commands.entity(player_entity).insert(ActiveBuff {
                    buff_type: BuffType::Speed,
                    remaining: 30.0,
                    magnitude: 1.5,
                });
            }
        }
        ItemType::StrengthPotion => {
            if inventory.remove_items(item, 1) {
                commands.entity(player_entity).insert(ActiveBuff {
                    buff_type: BuffType::Strength,
                    remaining: 30.0,
                    magnitude: 1.5,
                });
            }
        }
        _ => {}
    }
}

fn buff_tick(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ActiveBuff, &mut Health)>,
    time: Res<Time>,
) {
    for (entity, mut buff, mut health) in query.iter_mut() {
        // Regen buff heals over time
        if buff.buff_type == BuffType::Regen {
            health.heal(buff.magnitude * time.delta_secs());
        }
        buff.remaining -= time.delta_secs();
        if buff.remaining <= 0.0 {
            commands.entity(entity).remove::<ActiveBuff>();
        }
    }
}

#[derive(Resource, Default)]
pub struct ArmorSlots {
    pub helmet: Option<ItemType>,
    pub chest: Option<ItemType>,
    pub shield: Option<ItemType>,
}

impl ArmorSlots {
    pub fn total_armor(&self) -> u32 {
        let h = self.helmet.map(|i| i.armor_value()).unwrap_or(0);
        let c = self.chest.map(|i| i.armor_value()).unwrap_or(0);
        let s = self.shield.map(|i| i.shield_value()).unwrap_or(0);
        h + c + s
    }
}

fn equip_armor(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut armor: ResMut<ArmorSlots>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    let Some(slot) = inventory.selected_item() else { return };
    let item = slot.item;

    if item.is_helmet() {
        if inventory.remove_items(item, 1) {
            // Unequip old helmet back to inventory
            if let Some(old) = armor.helmet.take() {
                inventory.add_item(old, 1);
            }
            armor.helmet = Some(item);
        }
    } else if item.is_chestplate() {
        if inventory.remove_items(item, 1) {
            if let Some(old) = armor.chest.take() {
                inventory.add_item(old, 1);
            }
            armor.chest = Some(item);
        }
    } else if item.is_shield() {
        if inventory.remove_items(item, 1) {
            if let Some(old) = armor.shield.take() {
                inventory.add_item(old, 1);
            }
            armor.shield = Some(item);
        }
    }
}

/// Ticks the DamageFlash timer and tints the player sprite red while active.
fn update_damage_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &mut DamageFlash, &mut Sprite), With<Player>>,
    chunk_query: Query<&Chunk>,
) {
    for (entity, tf, mut flash, mut sprite) in query.iter_mut() {
        flash.timer -= time.delta_secs();
        if flash.timer > 0.0 {
            // Red tint while flashing
            sprite.color = Color::srgba(1.0, 0.3, 0.3, 1.0);
        } else {
            // Restore color: respect water tint if player is in water
            let in_water = is_position_water(tf.translation.x, tf.translation.y, &chunk_query);
            if in_water {
                sprite.color = Color::srgba(0.7, 0.8, 1.0, 0.85);
            } else {
                sprite.color = Color::WHITE;
            }
            commands.entity(entity).remove::<DamageFlash>();
        }
    }
}

/// Ticks the AttackLunge timer and scales the player sprite up briefly.
fn update_attack_lunge(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AttackLunge, &mut Sprite), With<Player>>,
) {
    for (entity, mut lunge, mut sprite) in query.iter_mut() {
        lunge.timer -= time.delta_secs();
        if lunge.timer > 0.0 {
            // Scale up by 10%
            sprite.custom_size = Some(Vec2::new(PLAYER_SIZE * 1.1, PLAYER_SIZE * 1.1));
        } else {
            // Reset to normal size (walk bob will override next frame if moving)
            sprite.custom_size = Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE));
            commands.entity(entity).remove::<AttackLunge>();
        }
    }
}
