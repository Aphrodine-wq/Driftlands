use bevy::prelude::*;
use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::hud::not_paused;
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::combat::{Enemy, EnemyType};
use crate::building::BuildingState;

// ---------------------------------------------------------------------------
// Pet type enum
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PetType {
    /// Tamed from FeralWolf. Attacks enemies, +15% player damage bonus.
    Wolf,
    /// Tamed from CaveSpider. Highlights nearby resources (cosmetic glow).
    Cat,
    /// Tamed from NightBat. Reveals larger minimap radius.
    Hawk,
    /// Tamed from BogLurker. Draws aggro, high HP tank.
    Bear,
}

impl PetType {
    /// The enemy type that can be tamed into this pet.
    pub fn source_enemy(&self) -> EnemyType {
        match self {
            PetType::Wolf => EnemyType::FeralWolf,
            PetType::Cat  => EnemyType::CaveSpider,
            PetType::Hawk => EnemyType::NightBat,
            PetType::Bear => EnemyType::BogLurker,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            PetType::Wolf => "Wolf",
            PetType::Cat  => "Cat",
            PetType::Hawk => "Hawk",
            PetType::Bear => "Bear",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            PetType::Wolf => Color::srgb(0.55, 0.55, 0.55),
            PetType::Cat  => Color::srgb(0.9, 0.55, 0.2),
            PetType::Hawk => Color::srgb(0.55, 0.35, 0.15),
            PetType::Bear => Color::srgb(0.15, 0.4, 0.15),
        }
    }

    pub fn size(&self) -> Vec2 {
        match self {
            PetType::Wolf => Vec2::new(10.0, 8.0),
            PetType::Cat  => Vec2::new(8.0, 8.0),
            PetType::Hawk => Vec2::new(8.0, 6.0),
            PetType::Bear => Vec2::new(12.0, 12.0),
        }
    }

    pub fn max_happiness(&self) -> f32 {
        100.0
    }

    pub fn follow_offset(&self) -> Vec2 {
        match self {
            PetType::Wolf => Vec2::new(-20.0, -10.0),
            PetType::Cat  => Vec2::new(15.0, -8.0),
            PetType::Hawk => Vec2::new(0.0, 20.0),
            PetType::Bear => Vec2::new(-25.0, -15.0),
        }
    }

    /// Returns the per-hit attack damage for combat-capable pets.
    fn attack_damage(&self) -> f32 {
        match self {
            PetType::Wolf => 5.0,
            PetType::Bear => 3.0,
            _ => 0.0,
        }
    }

    /// Whether this pet type can attack enemies.
    pub fn is_combat_pet(&self) -> bool {
        matches!(self, PetType::Wolf | PetType::Bear)
    }

    /// Try to derive a `PetType` from an enemy type (for taming).
    pub fn from_enemy(enemy_type: EnemyType) -> Option<PetType> {
        match enemy_type {
            EnemyType::FeralWolf  => Some(PetType::Wolf),
            EnemyType::CaveSpider => Some(PetType::Cat),
            EnemyType::NightBat   => Some(PetType::Hawk),
            EnemyType::BogLurker  => Some(PetType::Bear),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Pet {
    pub pet_type: PetType,
    pub happiness: f32,
    pub attack_cooldown: f32,
}

/// Serializable pet data for save/load.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PetData {
    pub pet_type_name: String,
    pub happiness: f32,
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct PetSystem {
    /// True when a pet entity currently exists in the world.
    pub active_pet: bool,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TAME_RANGE: f32 = 32.0;
const TAME_HEALTH_THRESHOLD: f32 = 0.30;
const TAME_SUCCESS_CHANCE: f64 = 0.50;
const FEED_RANGE: f32 = 32.0;
const FEED_HAPPINESS_RESTORE: f32 = 25.0;
const HAPPINESS_DECAY_PER_SEC: f32 = 1.0 / 60.0;
const FOLLOW_SMOOTH_SPEED: f32 = 3.0;
const ATTACK_RANGE: f32 = 80.0;
const ATTACK_HIT_RANGE: f32 = 16.0;
const ATTACK_MOVE_SPEED: f32 = 80.0;
const ATTACK_COOLDOWN_SECS: f32 = 1.5;
const ATTACK_HAPPINESS_MIN: f32 = 30.0;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PetPlugin;

impl Plugin for PetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PetSystem::default())
            .add_systems(Update, (
                attempt_tame,
                pet_follow,
                pet_happiness_decay,
                feed_pet,
                pet_attack,
                pet_despawn_check,
            ).run_if(not_paused));
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Right-click with PetCollar selected near a low-health tameable enemy to
/// attempt taming. Requires PetFood in inventory. 50% success chance.
fn attempt_tame(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    mut pet_system: ResMut<PetSystem>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Enemy)>,
    assets: Res<crate::assets::GameAssets>,
) {
    if !mouse.just_pressed(MouseButton::Right) || building_state.active {
        return;
    }

    // Must not already have a pet.
    if pet_system.active_pet {
        return;
    }

    // Selected hotbar item must be a PetCollar.
    let Some(slot) = inventory.selected_item() else { return };
    if slot.item != ItemType::PetCollar {
        return;
    }

    // Player must also carry PetFood.
    if !inventory.has_items(ItemType::PetFood, 1) {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    // Find the closest tameable enemy within range that is below the HP threshold.
    let mut best: Option<(Entity, Vec2, PetType)> = None;
    let mut best_dist = f32::MAX;

    for (entity, enemy_tf, enemy) in enemy_query.iter() {
        let Some(pet_type) = PetType::from_enemy(enemy.enemy_type) else { continue };

        // Must be below 30% health.
        if enemy.max_health <= 0.0 {
            continue;
        }
        let hp_frac = enemy.health / enemy.max_health;
        if hp_frac > TAME_HEALTH_THRESHOLD {
            continue;
        }

        let dist = player_pos.distance(enemy_tf.translation.truncate());
        if dist < TAME_RANGE && dist < best_dist {
            best_dist = dist;
            best = Some((entity, enemy_tf.translation.truncate(), pet_type));
        }
    }

    let Some((enemy_entity, spawn_pos, pet_type)) = best else { return };

    // Always consume the collar.
    inventory.remove_items(ItemType::PetCollar, 1);

    // Roll for tame success.
    let mut rng = rand::thread_rng();
    if rng.gen_bool(TAME_SUCCESS_CHANCE) {
        // Success: also consume PetFood.
        inventory.remove_items(ItemType::PetFood, 1);

        // Despawn the enemy.
        commands.entity(enemy_entity).despawn_recursive();

        // Spawn a pet entity with real sprite.
        let pet_image = match pet_type {
            PetType::Wolf => assets.pet_wolf.clone(),
            PetType::Cat => assets.pet_cat.clone(),
            PetType::Hawk => assets.pet_hawk.clone(),
            PetType::Bear => assets.pet_bear.clone(),
        };
        commands.spawn((
            Pet {
                pet_type,
                happiness: pet_type.max_happiness(),
                attack_cooldown: 0.0,
            },
            Sprite {
                image: pet_image,
                custom_size: Some(pet_type.size()),
                ..default()
            },
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, 6.0),
        ));

        pet_system.active_pet = true;
    }
    // On failure the collar is consumed but nothing else happens.
}

/// Pet smoothly follows the player at its type-specific offset.
fn pet_follow(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Pet>)>,
    mut pet_query: Query<(&Pet, &mut Transform), Without<Player>>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (pet, mut pet_tf) in pet_query.iter_mut() {
        let target = player_pos + pet.pet_type.follow_offset();
        let current = pet_tf.translation.truncate();
        let new_pos = current.lerp(target, FOLLOW_SMOOTH_SPEED * time.delta_secs());
        pet_tf.translation.x = new_pos.x;
        pet_tf.translation.y = new_pos.y;
    }
}

/// Happiness decreases slowly over time. When it hits zero the pet runs away.
fn pet_happiness_decay(
    mut commands: Commands,
    time: Res<Time>,
    mut pet_system: ResMut<PetSystem>,
    mut pet_query: Query<(Entity, &mut Pet)>,
) {
    for (entity, mut pet) in pet_query.iter_mut() {
        pet.happiness -= HAPPINESS_DECAY_PER_SEC * time.delta_secs();

        if pet.happiness <= 0.0 {
            commands.entity(entity).despawn_recursive();
            pet_system.active_pet = false;
        }
    }
}

/// Right-click with PetFood selected near the pet to feed it and restore happiness.
fn feed_pet(
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    mut inventory: ResMut<Inventory>,
    player_query: Query<&Transform, With<Player>>,
    mut pet_query: Query<(&mut Pet, &Transform), Without<Player>>,
) {
    if !mouse.just_pressed(MouseButton::Right) || building_state.active {
        return;
    }

    let Some(slot) = inventory.selected_item() else { return };
    if slot.item != ItemType::PetFood {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (mut pet, pet_tf) in pet_query.iter_mut() {
        let dist = player_pos.distance(pet_tf.translation.truncate());
        if dist < FEED_RANGE {
            if inventory.remove_items(ItemType::PetFood, 1) {
                let max = pet.pet_type.max_happiness();
                pet.happiness = (pet.happiness + FEED_HAPPINESS_RESTORE).min(max);
            }
            return;
        }
    }
}

/// Combat pets (Wolf, Bear) attack nearby enemies when happy enough.
fn pet_attack(
    time: Res<Time>,
    mut pet_query: Query<(&mut Pet, &mut Transform), Without<Enemy>>,
    mut enemy_query: Query<(Entity, &mut Enemy, &Transform), Without<Pet>>,
) {
    for (mut pet, mut pet_tf) in pet_query.iter_mut() {
        // Tick the cooldown regardless of combat state.
        if pet.attack_cooldown > 0.0 {
            pet.attack_cooldown -= time.delta_secs();
        }

        if !pet.pet_type.is_combat_pet() {
            continue;
        }
        if pet.happiness <= ATTACK_HAPPINESS_MIN {
            continue;
        }

        let pet_pos = pet_tf.translation.truncate();

        // Find the closest enemy within attack acquisition range.
        let mut closest: Option<(Entity, f32)> = None;
        let mut closest_dist = f32::MAX;

        for (entity, _enemy, enemy_tf) in enemy_query.iter() {
            let dist = pet_pos.distance(enemy_tf.translation.truncate());
            if dist < ATTACK_RANGE && dist < closest_dist {
                closest_dist = dist;
                closest = Some((entity, dist));
            }
        }

        let Some((target_entity, dist)) = closest else { continue };

        // Move toward the target enemy.
        let Ok((_entity, _enemy, enemy_tf)) = enemy_query.get(target_entity) else {
            continue;
        };
        let enemy_pos = enemy_tf.translation.truncate();
        let direction = (enemy_pos - pet_pos).normalize_or_zero();
        let move_delta = direction * ATTACK_MOVE_SPEED * time.delta_secs();
        pet_tf.translation.x += move_delta.x;
        pet_tf.translation.y += move_delta.y;

        // Attack if within melee range and cooldown is ready.
        if dist <= ATTACK_HIT_RANGE && pet.attack_cooldown <= 0.0 {
            if let Ok((_entity, mut enemy, _tf)) = enemy_query.get_mut(target_entity) {
                enemy.health -= pet.pet_type.attack_damage();
                pet.attack_cooldown = ATTACK_COOLDOWN_SECS;
            }
        }
    }
}

/// Sync the PetSystem resource when the pet entity is gone (killed, despawned, etc.).
fn pet_despawn_check(
    mut pet_system: ResMut<PetSystem>,
    pet_query: Query<&Pet>,
) {
    if pet_system.active_pet && pet_query.is_empty() {
        pet_system.active_pet = false;
    }
}
