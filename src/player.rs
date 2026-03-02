use bevy::prelude::*;
use crate::building::BuildingState;
use crate::inventory::{Inventory, ItemType};
use crate::world::TILE_SIZE;

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
            ));
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

        // Apply speed buff if present (ActiveBuff is attached to the player entity)
        let speed_buff = buffs_query.iter()
            .find(|b| b.buff_type == BuffType::Speed)
            .map(|b| b.magnitude)
            .unwrap_or(1.0);

        let speed = player.speed * hunger_multiplier * speed_buff;
        transform.translation.x += direction.x * speed * time.delta_secs();
        transform.translation.y += direction.y * speed * time.delta_secs();
    }
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
}

impl ArmorSlots {
    pub fn total_armor(&self) -> u32 {
        let h = self.helmet.map(|i| i.armor_value()).unwrap_or(0);
        let c = self.chest.map(|i| i.armor_value()).unwrap_or(0);
        h + c
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
    }
}
