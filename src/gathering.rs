use bevy::prelude::*;
use crate::world::{WorldObject, WorldObjectType};
use crate::inventory::{Inventory, ItemType};
use crate::player::Player;
use crate::building::BuildingState;
use crate::combat::Enemy;

pub struct GatheringPlugin;

impl Plugin for GatheringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, gather_resources);
    }
}

const GATHER_RANGE: f32 = 32.0;

fn gather_resources(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    building_state: Res<BuildingState>,
    player_query: Query<&Transform, With<Player>>,
    mut objects_query: Query<(Entity, &Transform, &mut WorldObject)>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut inventory: ResMut<Inventory>,
    time: Res<Time>,
) {
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

    if let Ok((_, _, mut object)) = objects_query.get_mut(target_entity) {
        object.health -= 30.0 * time.delta_secs();

        if object.health <= 0.0 {
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
                }
                WorldObjectType::AlpineFlower => {
                    inventory.add_item(ItemType::AlpineHerb, 1);
                }
                WorldObjectType::IronVein => {
                    inventory.add_item(ItemType::IronOre, 2);
                    inventory.add_item(ItemType::Stone, 1);
                }
                WorldObjectType::CoalDeposit => {
                    inventory.add_item(ItemType::Coal, 2);
                    inventory.add_item(ItemType::Stone, 1);
                }
            }
            // Consume tool durability
            inventory.use_selected_tool();
            commands.entity(target_entity).despawn();
        }
    }
}
