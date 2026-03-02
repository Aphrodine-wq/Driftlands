use bevy::prelude::*;
use crate::building::BuildingState;
use crate::inventory::{Inventory, ItemType};
use crate::world::TILE_SIZE;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (
                player_movement,
                hunger_depletion,
                starvation_damage,
                health_regeneration,
                eat_food,
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
        let speed_multiplier = if hunger.is_slow() { 0.7 } else { 1.0 };
        let speed = player.speed * speed_multiplier;
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
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    mut query: Query<&mut Hunger, With<Player>>,
) {
    if !mouse.just_pressed(MouseButton::Right) || building_state.active {
        return;
    }
    let Ok(mut hunger) = query.get_single_mut() else { return };

    let Some(slot) = inventory.selected_item() else { return };
    let item = slot.item;

    // Map each food item to (item, hunger_restored).
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
    }
}
