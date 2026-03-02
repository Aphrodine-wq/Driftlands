use bevy::prelude::*;
use crate::player::{Player, Health, Hunger};
use crate::inventory::{Inventory, InventorySlot};
use crate::world::TILE_SIZE;

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (check_player_death, recover_death_marker));
    }
}

#[derive(Component)]
pub struct DeathMarker {
    pub items: Vec<InventorySlot>,
}

fn check_player_death(
    mut commands: Commands,
    mut player_query: Query<(&mut Health, &mut Hunger, &mut Transform), With<Player>>,
    mut inventory: ResMut<Inventory>,
    existing_markers: Query<Entity, With<DeathMarker>>,
) {
    let Ok((mut health, mut hunger, mut transform)) = player_query.get_single_mut() else {
        return;
    };

    if !health.is_dead() {
        return;
    }

    let death_pos = transform.translation;

    // Collect all items from inventory
    let mut dropped_items = Vec::new();
    for slot in inventory.slots.iter_mut() {
        if let Some(s) = slot.take() {
            dropped_items.push(s);
        }
    }

    // Remove old death marker (only one at a time)
    for entity in existing_markers.iter() {
        commands.entity(entity).despawn();
    }

    // Spawn new death marker at death position
    if !dropped_items.is_empty() {
        commands.spawn((
            DeathMarker { items: dropped_items },
            Sprite {
                color: Color::srgb(1.0, 1.0, 0.2),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            Transform::from_xyz(death_pos.x, death_pos.y, 8.0),
        ));
    }

    // Respawn at world origin with full stats
    transform.translation = Vec3::new(TILE_SIZE * 16.0, TILE_SIZE * 16.0, 10.0);
    health.current = health.max;
    hunger.current = hunger.max;
    hunger.starvation_timer = 0.0;
}

fn recover_death_marker(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    marker_query: Query<(Entity, &Transform, &DeathMarker), Without<Player>>,
    mut inventory: ResMut<Inventory>,
) {
    let Ok(player_tf) = player_query.get_single() else { return };
    let player_pos = player_tf.translation.truncate();

    for (entity, tf, marker) in marker_query.iter() {
        let dist = player_pos.distance(tf.translation.truncate());
        if dist <= 24.0 {
            // Recover all items
            for item in &marker.items {
                inventory.add_item(item.item, item.count);
            }
            commands.entity(entity).despawn();
        }
    }
}
