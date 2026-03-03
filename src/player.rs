use bevy::prelude::*;
use crate::building::{Building, BuildingState, BuildingType, Door};
use crate::hud::not_paused;
use crate::inventory::{Inventory, ItemType};
use crate::world::{TILE_SIZE, CHUNK_WORLD_SIZE};
use crate::world::chunk::{Chunk, CHUNK_SIZE};

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
            ).run_if(not_paused));
    }
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
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
const PLAYER_SIZE: f32 = 12.0;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player { speed: PLAYER_SPEED },
        Health::new(100.0),
        Hunger::new(100.0),
        Sprite {
            color: Color::srgb(0.2, 0.4, 0.9),
            custom_size: Some(Vec2::new(PLAYER_SIZE, PLAYER_SIZE)),
            ..default()
        },
        Transform::from_xyz(
            TILE_SIZE * 16.0,
            TILE_SIZE * 16.0,
            10.0,
        ),
    ));
}

fn player_movement(
    mut query: Query<(&Player, &Hunger, &mut Transform)>,
    buffs_query: Query<&ActiveBuff>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    chunk_query: Query<&Chunk>,
    building_query: Query<(&Transform, &Building, Option<&Door>), Without<Player>>,
) {
    let Ok((player, hunger, mut transform)) = query.get_single_mut() else { return };

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        let hunger_multiplier = if hunger.is_slow() { 0.7 } else { 1.0 };

        let speed_buff = buffs_query.iter()
            .find(|b| b.buff_type == BuffType::Speed)
            .map(|b| b.magnitude)
            .unwrap_or(1.0);

        let speed = player.speed * hunger_multiplier * speed_buff;
        let delta = direction * speed * time.delta_secs();

        // Check X movement
        let target_x = transform.translation.x + delta.x;
        if is_position_walkable(target_x, transform.translation.y, &chunk_query)
            && !is_blocked_by_building(target_x, transform.translation.y, &building_query)
        {
            transform.translation.x = target_x;
        }

        // Check Y movement independently (allows sliding along walls)
        let target_y = transform.translation.y + delta.y;
        if is_position_walkable(transform.translation.x, target_y, &chunk_query)
            && !is_blocked_by_building(transform.translation.x, target_y, &building_query)
        {
            transform.translation.y = target_y;
        }
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

fn is_blocked_by_building(
    x: f32,
    y: f32,
    building_query: &Query<(&Transform, &Building, Option<&Door>), Without<Player>>,
) -> bool {
    let player_half = 6.0; // Half of PLAYER_SIZE (12/2)
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
        ItemType::CookedBerry => Some(25.0),
        ItemType::BakedWheat => Some(30.0),
        ItemType::CookedCarrot => Some(28.0),
        _ => None,
    };

    if let Some(value) = food_value {
        if inventory.remove_items(item, 1) {
            hunger.eat(value);
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
    mut query: Query<(Entity, &mut ActiveBuff)>,
    time: Res<Time>,
) {
    for (entity, mut buff) in query.iter_mut() {
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
